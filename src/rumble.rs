use gilrs::{
    Gilrs,
    ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Replay, Ticks},
};
use std::time::{Duration, Instant};
use thiserror::Error;

const TEST_DURATION: Duration = Duration::from_millis(300);
const TEST_MAGNITUDE: u16 = 32_000;

pub struct RumbleTest {
    effect: Effect,
    device_id: usize,
    ends_at: Instant,
}

impl RumbleTest {
    /// Starts a short, one-shot rumble effect on the selected controller.
    ///
    /// # Errors
    ///
    /// Returns a recoverable error if no suitable controller is selected or the
    /// backend cannot create or play the effect.
    pub fn start(gilrs: &mut Gilrs, selected_id: Option<usize>) -> Result<Self, RumbleError> {
        let selected_id = selected_id.ok_or(RumbleError::NoSelection)?;
        let (gamepad_id, connected, supported) = gilrs
            .gamepads()
            .find(|(id, _)| usize::from(*id) == selected_id)
            .map(|(id, gamepad)| (id, gamepad.is_connected(), gamepad.is_ff_supported()))
            .ok_or(RumbleError::Disconnected)?;

        if !connected {
            return Err(RumbleError::Disconnected);
        }
        if !supported {
            return Err(RumbleError::Unsupported);
        }

        let ticks = Ticks::from_ms(TEST_DURATION.as_millis().try_into().unwrap_or(u32::MAX));
        let scheduling = Replay {
            play_for: ticks,
            ..Replay::default()
        };
        let effect = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong {
                    magnitude: TEST_MAGNITUDE,
                },
                scheduling,
                ..BaseEffect::default()
            })
            .add_effect(BaseEffect {
                kind: BaseEffectType::Weak {
                    magnitude: TEST_MAGNITUDE,
                },
                scheduling,
                ..BaseEffect::default()
            })
            .gamepads(&[gamepad_id])
            .repeat(Repeat::For(ticks))
            .finish(gilrs)?;
        effect.play()?;

        Ok(Self {
            effect,
            device_id: selected_id,
            ends_at: Instant::now() + TEST_DURATION,
        })
    }

    #[must_use]
    pub fn device_id(&self) -> usize {
        self.device_id
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        Instant::now() >= self.ends_at
    }

    /// Stops the test before its bounded duration elapses.
    ///
    /// # Errors
    ///
    /// Returns an error if the force-feedback worker is unavailable.
    pub fn cancel(&self) -> Result<(), RumbleError> {
        self.effect.stop()?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RumbleError {
    #[error("no controller is selected")]
    NoSelection,
    #[error("the selected controller is disconnected")]
    Disconnected,
    #[error("rumble is unavailable for this controller or backend")]
    Unsupported,
    #[error("the controller backend could not run the rumble test: {0}")]
    Backend(#[from] gilrs::ff::Error),
}
