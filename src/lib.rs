use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const KEY_COUNT: usize = 768;
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub delay_ms: u16,
    pub key_delays_ms: BTreeMap<u16, u16>,
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.delay_ms > 200
            || self
                .key_delays_ms
                .iter()
                .any(|(&k, &v)| k == 0 || k as usize >= KEY_COUNT || v > 200)
        {
            return Err("Delays must be 0–200 ms and keycodes 1–767");
        }
        Ok(())
    }
}
pub fn parse_config(text: &str) -> Result<Config, String> {
    let config: Config = serde_json::from_str(text).map_err(|e| e.to_string())?;
    config.validate().map_err(str::to_owned)?;
    Ok(config)
}

#[derive(Clone, Copy, Default)]
struct Key {
    last_us: Option<u64>,
    down: bool,
    blocked: bool,
}
pub struct Debouncer {
    keys: [Key; KEY_COUNT],
    delays_us: [u64; KEY_COUNT],
    pub dropped: u64,
}
impl Debouncer {
    pub fn new(config: &Config) -> Self {
        let mut delays_us = [u64::from(config.delay_ms) * 1000; KEY_COUNT];
        for (&key, &delay) in &config.key_delays_ms {
            if let Some(slot) = delays_us.get_mut(key as usize) {
                *slot = u64::from(delay) * 1000;
            }
        }
        Self {
            keys: [Key::default(); KEY_COUNT],
            delays_us,
            dropped: 0,
        }
    }
    pub fn block_held(&mut self, code: u16) {
        if let Some(key) = self.keys.get_mut(code as usize) {
            key.blocked = true;
        }
    }
    pub fn pressed(&self) -> impl Iterator<Item = u16> + '_ {
        self.keys
            .iter()
            .enumerate()
            .filter(|(_, k)| k.down)
            .map(|(i, _)| i as u16)
    }
    pub fn accept(&mut self, code: u16, value: i32, time_us: u64) -> bool {
        let Some(key) = self.keys.get_mut(code as usize) else {
            return false;
        };
        match value {
            1 => {
                // Chatter is a press too soon after this key's last press or
                // release. Measuring from the press alone lets a long hold use
                // up the window, so bounces right after release got through.
                if key.down || key.blocked {
                    return false;
                }
                let recent = key
                    .last_us
                    .and_then(|last| time_us.checked_sub(last))
                    .is_some_and(|elapsed| elapsed < self.delays_us[code as usize]);
                key.last_us = Some(time_us);
                if recent {
                    key.blocked = true;
                    self.dropped += 1;
                    return false;
                }
                key.down = true;
                true
            }
            0 => {
                // Suppressed releases also restart the timer, so a burst of
                // bounces stays blocked until the key goes quiet.
                key.last_us = Some(time_us);
                key.blocked = false;
                std::mem::take(&mut key.down)
            }
            2 => key.down,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn guard() -> Debouncer {
        Debouncer::new(&Config {
            delay_ms: 40,
            key_delays_ms: BTreeMap::new(),
        })
    }
    #[test]
    fn chatter_and_intentional_double() {
        let mut d = guard();
        assert!(d.accept(30, 1, 0));
        assert!(d.accept(30, 0, 10000));
        assert!(!d.accept(30, 1, 20000));
        assert!(!d.accept(30, 0, 30000));
        assert!(!d.accept(30, 1, 40000)); // 10 ms after the bounce's release
        assert!(!d.accept(30, 0, 45000));
        assert!(d.accept(30, 1, 85000)); // 40 ms of quiet: intentional
        assert!(d.accept(30, 0, 95000));
        assert_eq!(d.dropped, 2);
    }
    #[test]
    fn bounce_after_long_hold_is_chatter() {
        // Butterfly chatter usually lands just after release, so a long hold
        // must not use up the delay window.
        let mut d = guard();
        assert!(d.accept(30, 1, 0));
        assert!(d.accept(30, 0, 120000));
        assert!(!d.accept(30, 1, 125000));
        assert!(!d.accept(30, 0, 130000));
        assert_eq!(d.dropped, 1);
    }
    #[test]
    fn bounce_burst_keeps_blocking_until_quiet() {
        let mut d = guard();
        assert!(d.accept(30, 1, 0));
        assert!(d.accept(30, 0, 10000));
        // Each bounce is under 40 ms after the previous one, though the last
        // is well over 40 ms after the real press.
        for t in [30000, 50000, 70000, 90000] {
            assert!(!d.accept(30, 1, t));
            assert!(!d.accept(30, 0, t + 5000));
        }
        assert!(d.accept(30, 1, 140000));
    }
    #[test]
    fn chords_and_repeat() {
        let mut d = guard();
        for (k, v, t) in [
            (42, 1, 0),
            (30, 1, 1),
            (48, 1, 2),
            (30, 2, 500000),
            (30, 0, 600000),
            (42, 0, 600001),
            (48, 0, 600002),
        ] {
            assert!(d.accept(k, v, t));
        }
        assert_eq!(d.pressed().count(), 0);
    }
    #[test]
    fn suppressed_hold_cannot_repeat() {
        let mut d = guard();
        d.accept(30, 1, 0);
        d.accept(30, 0, 1);
        assert!(!d.accept(30, 1, 2));
        assert!(!d.accept(30, 2, 500000));
        assert!(!d.accept(30, 0, 600000));
        assert!(d.accept(30, 1, 700000));
    }
    #[test]
    fn held_at_attach() {
        let mut d = guard();
        d.block_held(42);
        assert!(!d.accept(42, 2, 0));
        assert!(!d.accept(42, 0, 1));
        assert!(d.accept(42, 1, 40001));
    }
    #[test]
    fn overrides_and_disabled() {
        let c = Config {
            delay_ms: 40,
            key_delays_ms: BTreeMap::from([(30, 0), (48, 80)]),
        };
        let mut d = Debouncer::new(&c);
        for key in [30, 48] {
            assert!(d.accept(key, 1, 0));
            assert!(d.accept(key, 0, 1));
        }
        assert!(d.accept(30, 1, 2));
        assert!(!d.accept(48, 1, 60000));
    }
    #[test]
    fn clock_backwards_is_not_chatter() {
        let mut d = guard();
        d.accept(30, 1, 100000);
        d.accept(30, 0, 100001);
        assert!(d.accept(30, 1, 1));
    }
    #[test]
    fn invalid_configuration() {
        for text in [
            r#"{"delay_ms":201,"key_delays_ms":{}}"#,
            r#"{"delay_ms":40,"key_delays_ms":{"768":40}}"#,
        ] {
            assert!(serde_json::from_str::<Config>(text)
                .unwrap()
                .validate()
                .is_err());
        }
        assert!(serde_json::from_str::<Config>(r#"{"delay_ms":-1,"key_delays_ms":{}}"#).is_err());
    }
    #[test]
    fn randomized_transitions_stay_balanced() {
        let mut d = guard();
        let mut visible = [false; KEY_COUNT];
        let mut rng = 42u64;
        for time in 0..100000 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let code = ((rng >> 32) % 10 + 30) as u16;
            let value = ((rng >> 16) % 3) as i32;
            if d.accept(code, value, time * 100) {
                let down = &mut visible[code as usize];
                match value {
                    1 => {
                        assert!(!*down);
                        *down = true;
                    }
                    0 => {
                        assert!(*down);
                        *down = false;
                    }
                    _ => assert!(*down),
                }
            }
        }
        assert_eq!(
            d.pressed().collect::<Vec<_>>(),
            visible
                .iter()
                .enumerate()
                .filter(|(_, v)| **v)
                .map(|(i, _)| i as u16)
                .collect::<Vec<_>>()
        );
    }
}
