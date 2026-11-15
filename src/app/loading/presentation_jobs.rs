//! Display-only loading jobs. Native Rules, the simulation and the process
//! asset lease remain with the loading attempt; workers own only its derived
//! physical-source projections. No frame is installed before both atlases finish.

use super::{PreparedManifest, PresentationManifest};
use crate::app::loading::cpu_job::CpuLoadJob;
use crate::render::sidebar_cameo_atlas::{PreparedSidebarCameos, SidebarCameoWork};
use crate::render::{batch::BatchRenderer, gpu::GpuContext, sprite_atlas, unit_atlas};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::task::Poll;

pub(crate) struct PresentationAtlasJob {
    unit: Option<(
        mpsc::Receiver<unit_atlas::UnitRasterOutput>,
        unit_atlas::UnitAtlasUpload,
    )>,
    sprite: Option<sprite_atlas::SpriteAtlasLoadingJob>,
    cameos: Option<CpuLoadJob<Option<PreparedSidebarCameos>>>,
    cancel: Arc<AtomicBool>,
}

impl PreparedManifest {
    /// Saved-game preparation retains its synchronous transaction. The same
    /// derived inputs and atlas owners also serve the yielding fresh-load path.
    pub(crate) fn finish(self, gpu: &GpuContext, batch: &BatchRenderer) -> PresentationManifest {
        PresentationManifest {
            unit_atlas: self.unit_work.and_then(|(work, upload)| {
                let mut atlas = None;
                upload.finish(&gpu.device, &gpu.queue, batch, work.render(), &mut atlas);
                atlas
            }),
            sprite_atlas: self
                .sprite_work
                .and_then(|work| work.finish(&gpu.device, &gpu.queue, batch)),
            palette_set: self.palette_set,
        }
    }

    pub(crate) fn start_loading(
        self,
        gpu: &GpuContext,
    ) -> anyhow::Result<(PresentationManifest, PresentationAtlasJob)> {
        let cancel = Arc::new(AtomicBool::new(false));
        let unit = match self.unit_work {
            Some((work, upload)) => Some((work.spawn(cancel.clone())?, upload)),
            None => None,
        };
        let sprite = match self
            .sprite_work
            .map(|work| work.start_loading(&gpu.device))
            .transpose()
        {
            Ok(sprite) => sprite,
            Err(error) => {
                cancel.store(true, Ordering::Relaxed);
                return Err(error);
            }
        };
        Ok((
            PresentationManifest {
                unit_atlas: None,
                sprite_atlas: None,
                palette_set: self.palette_set,
            },
            PresentationAtlasJob {
                unit,
                sprite,
                cameos: None,
                cancel,
            },
        ))
    }
}

impl PresentationAtlasJob {
    pub(crate) fn defer_sidebar_cameos(&mut self, work: SidebarCameoWork) -> anyhow::Result<()> {
        self.cameos = Some(CpuLoadJob::start("sidebar-cameos", move || work.render())?);
        Ok(())
    }

    pub(crate) fn poll(
        &mut self,
        gpu: &GpuContext,
        batch: &BatchRenderer,
        presentation: &mut crate::app::loading::init::PresentationLoadAssets,
    ) -> anyhow::Result<bool> {
        if let Some((receiver, _)) = &self.unit {
            match receiver.try_recv() {
                Ok(output) => {
                    let (_, upload) = self.unit.take().expect("one voxel completion owner");
                    upload.finish(
                        &gpu.device,
                        &gpu.queue,
                        batch,
                        output,
                        &mut presentation.unit_atlas,
                    );
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    anyhow::bail!("voxel loading worker disconnected")
                }
            }
        }
        if let Some(sprite) = &mut self.sprite {
            if let Poll::Ready(atlas) = sprite.poll(&gpu.device, &gpu.queue, batch)? {
                presentation.sprite_atlas = atlas;
                self.sprite = None;
            }
        }
        if let Some(cameos) = &mut self.cameos
            && let Poll::Ready(ready) = cameos.poll()?
        {
            presentation.sidebar_cameo_atlas =
                ready.map(|atlas| atlas.upload(&gpu.device, &gpu.queue, batch));
            self.cameos = None;
        }
        Ok(self.unit.is_none() && self.sprite.is_none() && self.cameos.is_none())
    }
}

impl Drop for PresentationAtlasJob {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
