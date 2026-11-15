//! Loading-only band decoding and nonblocking GPU retirement. Live growth
//! appends through [`ShpAtlasUpload::finish`]; both consume the parent atlas
//! owners.

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::task::Poll;

enum RasterMessage {
    Band {
        page: usize,
        origin: [u32; 2],
        size: [u32; 2],
        bytes: Vec<u8>,
    },
    Finished,
}

struct InFlightBand {
    started: Instant,
    complete: Arc<AtomicBool>,
}

/// Private candidate: it is never visible to a scene until every band retires.
struct LoadingUpload {
    completion: Option<ShpAtlasUpload>,
    unrenderable: Vec<ShpSpriteKey>,
    atlas: Option<SpriteAtlas>,
    indices: Vec<wgpu::Texture>,
    #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
    messages: mpsc::Receiver<RasterMessage>,
    #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
    retired: mpsc::Sender<()>,
    #[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
    messages: Box<dyn Iterator<Item = RasterMessage>>,
    in_flight: Option<InFlightBand>,
    added: usize,
    texture_limit: u32,
}

/// Everything below GPU allocation is owned by the CPU worker. Only this
/// private transfer, never a GPU resource, crosses back to the window thread.
struct PackedLoadingAtlas {
    completion: ShpAtlasUpload,
    unrenderable: Vec<ShpSpriteKey>,
    plan: ShpAtlasPackPlan,
    palettes: ShpPalettes,
    added: usize,
    texture_limit: u32,
}

enum LoadingStage {
    Preparing(mpsc::Receiver<anyhow::Result<Option<PackedLoadingAtlas>>>),
    Uploading(Box<LoadingUpload>),
    Empty,
    Consumed,
}

pub(crate) struct SpriteAtlasLoadingJob(LoadingStage);

impl PreparedSpriteAtlas {
    pub(crate) fn start_loading(
        self,
        device: &wgpu::Device,
    ) -> anyhow::Result<SpriteAtlasLoadingJob> {
        let (work, completion) = match self {
            Self::Unchanged => return Ok(SpriteAtlasLoadingJob(LoadingStage::Empty)),
            Self::Raster(work, upload) => (work, upload),
        };
        anyhow::ensure!(completion.initial, "loading must not replace a live atlas");
        let texture_limit = device.limits().max_texture_dimension_2d;
        let (send, receive) = mpsc::channel();
        let prepare = move || {
            let _ = send.send(PackedLoadingAtlas::prepare(work, completion, texture_limit));
        };
        #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
        std::thread::Builder::new()
            .name("shp-loading-prepare".into())
            .spawn(prepare)?;
        #[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
        prepare();
        Ok(SpriteAtlasLoadingJob(LoadingStage::Preparing(receive)))
    }
}

impl PackedLoadingAtlas {
    fn prepare(
        work: ShpRasterWork,
        completion: ShpAtlasUpload,
        texture_limit: u32,
    ) -> anyhow::Result<Option<Self>> {
        // Fail the loading attempt with the raster's actual diagnostic. No
        // candidate has been published, so the caller can retire its lease.
        let (rendered, unrenderable, palettes) = match work.render().0 {
            ShpRaster::Drawn {
                rendered,
                unrenderable,
                rows,
            } => (rendered, unrenderable, rows),
            ShpRaster::Abandoned(failure) => anyhow::bail!("{failure}"),
        };
        let added = rendered.len();
        if rendered.is_empty() {
            log::warn!("No SHP sprites rendered");
            return Ok(None);
        }
        let plan = plan_shp_pages(rendered, texture_limit);
        Ok(Some(Self {
            completion,
            unrenderable,
            plan,
            palettes,
            added,
            texture_limit,
        }))
    }

    fn begin_upload(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batch: &BatchRenderer,
    ) -> anyhow::Result<LoadingUpload> {
        let Self {
            completion,
            unrenderable,
            mut plan,
            palettes,
            added,
            texture_limit,
        } = self;
        let (atlas, indices) = create_packed_shp_atlas(device, queue, batch, &mut plan, palettes);
        #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
        let (send, messages) = mpsc::sync_channel(1);
        #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
        let (retired, wait_retired) = mpsc::channel();
        #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
        std::thread::Builder::new()
            .name("shp-loading-bands".into())
            .spawn(move || {
                for (page, placement) in plan.pages.into_iter().enumerate() {
                    let bands = ShpUploadBands::new(
                        plan.sprites.clone(),
                        placement.placed,
                        placement.size[0],
                        PACK_BAND_BYTES,
                    );
                    for (origin, size, bytes) in bands {
                        if send
                            .send(RasterMessage::Band {
                                page,
                                origin,
                                size,
                                bytes,
                            })
                            .is_err()
                        {
                            return;
                        }
                        // Keep the existing one-band memory bound. The next
                        // band is decoded only after this staging copy retires.
                        // Dropping a cancelled job closes both channels; it
                        // never joins a worker or lends it process/GPU owners.
                        if wait_retired.recv().is_err() {
                            return;
                        }
                    }
                }
                let _ = send.send(RasterMessage::Finished);
            })?;
        #[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
        let messages: Box<dyn Iterator<Item = RasterMessage>> = {
            let sprites = plan.sprites;
            Box::new(
                plan.pages
                    .into_iter()
                    .enumerate()
                    .flat_map(move |(page, placement)| {
                        ShpUploadBands::new(
                            sprites.clone(),
                            placement.placed,
                            placement.size[0],
                            PACK_BAND_BYTES,
                        )
                        .map(move |(origin, size, bytes)| {
                            RasterMessage::Band {
                                page,
                                origin,
                                size,
                                bytes,
                            }
                        })
                    })
                    .chain(std::iter::once(RasterMessage::Finished)),
            )
        };
        Ok(LoadingUpload {
            completion: Some(completion),
            unrenderable,
            atlas: Some(atlas),
            indices,
            messages,
            #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
            retired,
            in_flight: None,
            added,
            texture_limit,
        })
    }
}

impl SpriteAtlasLoadingJob {
    /// At most one small queue write per event-loop pass, with no GPU wait.
    /// wgpu's completion callback is dispatched by Device::poll(Poll);
    /// unlike PollType::Wait this returns control to the window immediately.
    pub(crate) fn poll(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batch: &BatchRenderer,
    ) -> anyhow::Result<Poll<Option<SpriteAtlas>>> {
        if let LoadingStage::Preparing(result) = &self.0 {
            self.0 = match result.try_recv() {
                Ok(Ok(Some(atlas))) => {
                    LoadingStage::Uploading(Box::new(atlas.begin_upload(device, queue, batch)?))
                }
                Ok(Ok(None)) => LoadingStage::Empty,
                Ok(Err(error)) => return Err(error),
                Err(mpsc::TryRecvError::Empty) => return Ok(Poll::Pending),
                Err(mpsc::TryRecvError::Disconnected) => {
                    anyhow::bail!("SHP preparation worker disconnected")
                }
            };
        }
        let upload = match &mut self.0 {
            LoadingStage::Uploading(upload) => upload,
            LoadingStage::Empty => {
                self.0 = LoadingStage::Consumed;
                return Ok(Poll::Ready(None));
            }
            LoadingStage::Consumed => anyhow::bail!("SHP loading job already consumed"),
            LoadingStage::Preparing(_) => unreachable!("preparation consumed above"),
        };
        device.poll(wgpu::PollType::Poll)?;
        if let Some(band) = &upload.in_flight {
            if !band.complete.load(Ordering::Acquire) {
                anyhow::ensure!(
                    band.started.elapsed() < PACK_BAND_TIMEOUT,
                    "SHP loading upload timed out"
                );
                return Ok(Poll::Pending);
            }
            upload.in_flight = None;
            #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
            upload.retired.send(())?;
        }
        #[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
        let message = upload.messages.try_recv();
        #[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
        let message = upload
            .messages
            .next()
            .ok_or(mpsc::TryRecvError::Disconnected);
        match message {
            Ok(RasterMessage::Band {
                page,
                origin,
                size,
                bytes,
            }) => {
                atlas_growth::write_texels(queue, &upload.indices[page], origin, size, 1, &bytes);
                queue.submit(std::iter::empty());
                let complete = Arc::new(AtomicBool::new(false));
                let callback = complete.clone();
                queue.on_submitted_work_done(move || callback.store(true, Ordering::Release));
                upload.in_flight = Some(InFlightBand {
                    started: Instant::now(),
                    complete,
                });
                Ok(Poll::Pending)
            }
            Ok(RasterMessage::Finished) => {
                let mut atlas = upload.atlas.take().expect("unpublished loading candidate");
                log_shp_atlas_pack(&atlas, upload.texture_limit);
                upload
                    .completion
                    .take()
                    .expect("one completion owner")
                    .publish(
                        &mut atlas,
                        upload.added,
                        std::mem::take(&mut upload.unrenderable),
                    );
                self.0 = LoadingStage::Consumed;
                Ok(Poll::Ready(Some(atlas)))
            }
            Err(mpsc::TryRecvError::Empty) => Ok(Poll::Pending),
            Err(mpsc::TryRecvError::Disconnected) => {
                anyhow::bail!("SHP loading worker disconnected")
            }
        }
    }
}
