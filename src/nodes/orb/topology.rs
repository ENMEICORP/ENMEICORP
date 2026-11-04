//! THE BLACK FOREST // NODE 03 // CELESTIAL TOPOLOGY 
//! Doneware Invariance (Centennial 2098) - ENMEICORP

use super::math::{Vec3, slerp_sphere};

pub const NUM_STARS: usize = 20;
pub const SYNAPSE_SAMPLES: usize = 24;
pub const RING_SAMPLES: usize = 256;
pub const NUM_DUST_PARTICLES: usize = 1536;

const GOLDEN_ANGLE: f32 = 2.399_963_2;

#[derive(Debug, Clone, Copy)]
pub struct Star {
    pub id: usize,
    pub pos: Vec3,
    pub base_luminance: f32,
    pub phase: f32,
    pub is_vault: bool,
}

#[derive(Debug, Clone)]
pub struct SynapseArc {
    pub from_star: usize,
    pub to_star: usize,
    pub samples: [Vec3; SYNAPSE_SAMPLES],
    pub speed: f32,
    pub phase_offset: f32,
}

#[derive(Debug, Clone)]
pub struct OrbitalRing {
    pub radius: f32,
    pub base_samples: [Vec3; RING_SAMPLES],
    pub axis_tilt: Vec3,
    pub rot_speed: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DustParticle {
    pub pos: Vec3,
    pub intensity: f32,
    pub phase: f32,
    pub speed: f32,
    pub is_gold: bool,
}

#[derive(Debug, Clone)]
pub struct CelestialLattice {
    pub stars: [Star; NUM_STARS],
    pub synapses: Vec<SynapseArc>,
    pub rings: [OrbitalRing; 3],
    pub dust: [DustParticle; NUM_DUST_PARTICLES],
}

impl Default for CelestialLattice {
    fn default() -> Self {
        Self::new()
    }
}

impl CelestialLattice {
    pub fn new() -> Self {
        let stars = Self::generate_fibonacci_stars();
        let synapses = Self::generate_synapses(&stars);
        let rings = Self::generate_orbital_rings();
        let dust = Self::generate_internal_dust();

        Self {
            stars,
            synapses,
            rings,
            dust,
        }
    }

    fn generate_fibonacci_stars() -> [Star; NUM_STARS] {
        let mut stars = [Star {
            id: 0,
            pos: Vec3::ZERO,
            base_luminance: 1.0,
            phase: 0.0,
            is_vault: true,
        }; NUM_STARS];

        let n_f32 = NUM_STARS as f32;

        for i in 0..NUM_STARS {
            let i_f = i as f32;
            let y = 1.0 - (i_f / (n_f32 - 1.0)) * 2.0;
            let radius = (1.0 - y * y).max(0.0).sqrt();
            let theta = GOLDEN_ANGLE * i_f;

            let x = theta.cos() * radius;
            let z = theta.sin() * radius;

            // Luminance boost and white-gold spectral emission
            let luminance = 1.15 + 0.15 * ((i * 7 + 3) % 11) as f32 / 10.0;
            let phase = (i as f32 * 1.618033) % (std::f32::consts::PI * 2.0);

            stars[i] = Star {
                id: i,
                pos: Vec3::new(x, y, z).normalize(),
                base_luminance: luminance,
                phase,
                is_vault: true, // 100% of all 20 nodes designated Imperial / White Gold
            };
        }
        stars
    }

    fn generate_synapses(stars: &[Star; NUM_STARS]) -> Vec<SynapseArc> {
        let mut synapses = Vec::new();
        let mut idx = 0;

        for i in 0..NUM_STARS {
            for j in (i + 1)..NUM_STARS {
                let dot = stars[i].pos.dot(stars[j].pos);
                if dot > 0.30 && dot < 0.95 {
                    let mut samples = [Vec3::ZERO; SYNAPSE_SAMPLES];
                    for (s, sample) in samples.iter_mut().enumerate().take(SYNAPSE_SAMPLES) {
                        let t = s as f32 / (SYNAPSE_SAMPLES - 1) as f32;
                        *sample = slerp_sphere(stars[i].pos, stars[j].pos, t);
                    }
                    synapses.push(SynapseArc {
                        from_star: i,
                        to_star: j,
                        samples,
                        speed: 0.7 + ((idx % 7) as f32) * 0.18,
                        phase_offset: (idx as f32 * 0.431) % 1.0,
                    });
                    idx += 1;
                }
            }
        }
        synapses
    }

    fn generate_orbital_rings() -> [OrbitalRing; 3] {
        let step = (std::f32::consts::PI * 2.0) / (RING_SAMPLES as f32);

        let mut base_pts = [Vec3::ZERO; RING_SAMPLES];
        for (i, pt) in base_pts.iter_mut().enumerate().take(RING_SAMPLES) {
            let theta = i as f32 * step;
            *pt = Vec3::new(theta.cos(), 0.0, theta.sin());
        }

        [
            OrbitalRing {
                radius: 1.20,
                base_samples: base_pts,
                axis_tilt: Vec3::new(0.35, 0.92, 0.15).normalize(),
                rot_speed: 0.05,
            },
            OrbitalRing {
                radius: 1.32,
                base_samples: base_pts,
                axis_tilt: Vec3::new(-0.70, 0.40, 0.58).normalize(),
                rot_speed: -0.04,
            },
            OrbitalRing {
                radius: 1.46,
                base_samples: base_pts,
                axis_tilt: Vec3::new(0.28, 0.65, -0.70).normalize(),
                rot_speed: 0.03,
            },
        ]
    }

    fn generate_internal_dust() -> [DustParticle; NUM_DUST_PARTICLES] {
        let mut dust = [DustParticle {
            pos: Vec3::ZERO,
            intensity: 0.0,
            phase: 0.0,
            speed: 0.0,
            is_gold: true,
        }; NUM_DUST_PARTICLES];

        let mut state: u32 = 0x2098_CAFE;
        let mut next_f32 = || -> f32 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (state as f32) / (u32::MAX as f32)
        };

        for item in dust.iter_mut().take(NUM_DUST_PARTICLES) {
            let r = next_f32().cbrt() * 0.96;
            let theta = next_f32() * std::f32::consts::PI * 2.0;
            let phi = (next_f32() * 2.0 - 1.0).acos();

            let x = r * phi.sin() * theta.cos();
            let y = r * phi.cos();
            let z = r * phi.sin() * theta.sin();

            *item = DustParticle {
                pos: Vec3::new(x, y, z),
                intensity: 0.40 + 0.60 * next_f32(),
                phase: next_f32() * std::f32::consts::PI * 2.0,
                speed: 0.4 + next_f32() * 1.6,
                is_gold: true, 
            };
        }
        dust
    }
}