//! The game's gamma, which brightens textures and maps light the way the
//! engine's gamma tables do (`BuildGammaTable`, as Xash3D restates it)
//!
//! Four settings make two tables. The texture table raises every texture
//! colour to `texgamma / gamma`. The light table, 1024 steps long, takes
//! light to `lightgamma`, multiplies it by a `brightness` over 1, stretches
//! the darkest part by the brightness and takes the result to `1 / gamma`.
//! A lightmap reaches the table at four times its stored value, and either
//! is first raised by the light gamma's root of two, or with
//! `gl_overbright` is doubled after the table instead. A model's light, 0
//! to 255, reaches it at four times its value. The shaders work these out
//! for each texel, so changing a setting needs no textures sent again

/// The game's gamma settings, in the ranges the game allows
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gamma {
    /// `gamma`, 1.8 to 3
    pub gamma: f32,
    /// `brightness`, 0 to 3
    pub brightness: f32,
    /// `texgamma`, 1.8 to 3
    pub texgamma: f32,
    /// `lightgamma`, 1.8 to 3
    pub lightgamma: f32,
    /// `gl_overbright`
    pub overbright: bool,
}

impl Default for Gamma {
    /// The game's own defaults
    fn default() -> Self {
        Self {
            gamma: 2.5,
            brightness: 0.,
            texgamma: 2.,
            lightgamma: 2.5,
            overbright: false,
        }
    }
}

impl Gamma {
    /// Within the game's ranges, as it keeps its settings
    pub fn clamped(self) -> Self {
        Self {
            gamma: self.gamma.clamp(1.8, 3.),
            brightness: self.brightness.clamp(0., 3.),
            texgamma: self.texgamma.clamp(1.8, 3.),
            lightgamma: self.lightgamma.clamp(1.8, 3.),
            overbright: self.overbright,
        }
    }

    /// Where the light table's dark part ends, lower for more brightness
    fn knee(&self) -> f32 {
        if self.brightness <= 0. {
            0.125
        } else if self.brightness <= 1. {
            0.125 - self.brightness * self.brightness * 0.075
        } else {
            0.05
        }
    }

    /// A texture colour, 0 to 1, through the texture table
    pub fn texture(&self, colour: f32) -> f32 {
        let g = self.clamped();
        colour.powf(g.texgamma / g.gamma)
    }

    /// The light table at `x`, its place along the table from 0 to 1
    pub fn light(&self, x: f32) -> f32 {
        let g = self.clamped();
        let mut f = x.max(0.).powf(g.lightgamma);
        if g.brightness > 1. {
            f *= g.brightness;
        }
        let knee = g.knee();
        f = if f <= knee {
            f / knee * 0.125
        } else {
            (f - knee) / (1. - knee) * 0.875 + 0.125
        };
        f.powf(1. / g.gamma)
    }

    /// What a lightmap's light, 1 for a stored 255 under a normal style,
    /// makes of a texture's colour
    pub fn lightmap(&self, light: f32) -> f32 {
        let g = self.clamped();
        let raise = if g.overbright {
            1.
        } else {
            2f32.powf(1. / g.lightgamma)
        };
        let index = (light * 255. * 4. * raise).min(1023.);
        let doubled = if g.overbright { 2. } else { 1. };
        self.light(index / 1023.) * doubled
    }

    /// What a model's light, 0 to 1, makes of a texture's colour
    pub fn model(&self, light: f32) -> f32 {
        self.light((light.clamp(0., 1.) * 1020.) / 1023.)
    }

    /// The settings as the shaders take them, the texture table's power,
    /// the light gamma, one over gamma and the knee, then the brightness,
    /// the raise before the light table and the multiple after it
    pub fn uniform(&self) -> [[f32; 4]; 2] {
        let g = self.clamped();
        let (raise, doubled) = if g.overbright {
            (1., 2.)
        } else {
            (2f32.powf(1. / g.lightgamma), 1.)
        };
        [
            [g.texgamma / g.gamma, g.lightgamma, 1. / g.gamma, g.knee()],
            [g.brightness, raise, doubled, 0.],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Table entries as the engine stores them, cut to whole steps
    fn texture_entry(g: &Gamma, i: u32) -> u32 {
        (g.texture(i as f32 / 255.) * 255.) as u32
    }

    fn light_entry(g: &Gamma, i: u32) -> u32 {
        (g.light(i as f32 / 1023.) * 1023.) as u32
    }

    #[test]
    fn the_tables_match_the_engines_published_entries() {
        // Xash3D's test tables for the defaults, and for gamma 2.2,
        // brightness 1, texture gamma 2.2 and light gamma 2.4
        let defaults = Gamma::default();
        for (i, want) in [(10, 19), (100, 120), (200, 209)] {
            assert!(texture_entry(&defaults, i).abs_diff(want) <= 1, "{i}");
        }
        for (i, want) in [(100, 99), (300, 299), (500, 499), (900, 900)] {
            assert!(light_entry(&defaults, i).abs_diff(want) <= 1, "{i}");
        }
        let bright = Gamma {
            gamma: 2.2,
            brightness: 1.,
            texgamma: 2.2,
            lightgamma: 2.4,
            overbright: false,
        };
        for (i, want) in [(10, 10), (100, 100), (200, 200)] {
            assert!(texture_entry(&bright, i).abs_diff(want) <= 1, "{i}");
        }
        for (i, want) in [(100, 122), (300, 401), (500, 538), (900, 900)] {
            assert!(light_entry(&bright, i).abs_diff(want) <= 1, "{i}");
        }
    }

    #[test]
    fn overbright_doubles_after_the_table_and_settings_stay_in_range() {
        let plain = Gamma::default();
        let over = Gamma {
            overbright: true,
            ..plain
        };
        // With the defaults the light table is nearly straight, so a
        // half-lit lightmap is raised by about 1.32 or doubled
        assert!((plain.lightmap(0.5) - 0.5 * 2f32.powf(0.4)).abs() < 0.01);
        assert!((over.lightmap(0.5) - 2. * 510. / 1023.).abs() < 0.01);
        let wild = Gamma {
            gamma: 9.,
            brightness: -1.,
            texgamma: 0.,
            lightgamma: 1.,
            overbright: false,
        }
        .clamped();
        assert_eq!(
            (wild.gamma, wild.brightness, wild.texgamma, wild.lightgamma),
            (3., 0., 1.8, 1.8)
        );
    }
}
