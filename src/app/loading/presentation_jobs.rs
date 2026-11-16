//! Display-only loading jobs. Native Rules, the simulation and the process
//! asset lease remain with the loading attempt; workers own only its derived
//! physical-source projections. No frame is installed before the sprite atlas
//! finishes; the unit atlas draws its pictures on first use.

use super::{PreparedManifest, PresentationManifest};
use crate::app::loading::cpu_job::CpuLoadJob;
use crate::render::sidebar_cameo_atlas::{PreparedSidebarCameos, SidebarCameoWork};
use crate::render::{batch::BatchRenderer, gpu::GpuContext, sprite_atlas};
use std::task::Poll;

pub(crate) struct PresentationAtlasJob {
    sprite: Option<sprite_atlas::SpriteAtlasLoadingJob>,
    cameos: Option<CpuLoadJob<Option<PreparedSidebarCameos>>>,
}

impl PreparedManifest {
    /// Saved-game preparation retains its synchronous transaction. The same
    /// derived inputs and atlas owners also serve the yielding fresh-load path.
    pub(crate) fn finish(self, gpu: &GpuContext, batch: &BatchRenderer) -> PresentationManifest {
        PresentationManifest {
            unit_atlas: self.unit_atlas,
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
        let sprite = self
            .sprite_work
            .map(|work| work.start_loading(&gpu.device))
            .transpose()?;
        Ok((
            PresentationManifest {
                unit_atlas: self.unit_atlas,
                sprite_atlas: None,
                palette_set: self.palette_set,
            },
            PresentationAtlasJob {
                sprite,
                cameos: None,
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
        if let Some(sprite) = &mut self.sprite
            && let Poll::Ready(atlas) = sprite.poll(&gpu.device, &gpu.queue, batch)?
        {
            presentation.sprite_atlas = atlas;
            self.sprite = None;
        }
        if let Some(cameos) = &mut self.cameos
            && let Poll::Ready(ready) = cameos.poll()?
        {
            presentation.sidebar_cameo_atlas =
                ready.map(|atlas| atlas.upload(&gpu.device, &gpu.queue, batch));
            self.cameos = None;
        }
        Ok(self.sprite.is_none() && self.cameos.is_none())
    }
}
