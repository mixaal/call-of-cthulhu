use std::time::{Duration, Instant};

use rand::RngExt;

struct RainDrop {
    x: usize,
    y: usize,
    t: Instant,
    k: f32,
}

impl RainDrop {
    fn new(k: f32, x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            k,
            t: Instant::now(),
        }
    }
}

pub struct Rain {
    rain_drop_ttl: f32,      // time to live for the rain drop in seconds
    rain_drop_velocity: f32, // how often to add a new rain drop
    k: f32,
    pub(crate) width: usize,
    pub(crate) height: usize,
    rain_drops: Vec<RainDrop>,
    rng: rand::rngs::ThreadRng,
    last_drop_added: Instant,
}

impl Rain {
    pub fn new(width: usize, height: usize, rain_drop_ttl: f32, rain_drop_velocity: f32) -> Self {
        let k = 15.0;
        gen_wave_xy_lookup(k);
        Self {
            k,
            rain_drop_ttl,
            rain_drop_velocity,
            width,
            height,
            rain_drops: Vec::new(),
            rng: rand::rng(),
            last_drop_added: Instant::now()
                .checked_sub(Duration::from_secs(1000))
                .unwrap(),
        }
    }

    pub(crate) fn add_rain_drop_if_time_elapsed(&mut self) {
        if self.last_drop_added.elapsed().as_secs_f32() > self.rain_drop_velocity {
            self.rain_drops.push(RainDrop::new(
                self.k,
                self.rng.random_range(0..self.width),
                self.rng.random_range(0..self.height),
            ));
            self.last_drop_added = Instant::now();
        }
    }

    pub(crate) fn remove_rain_drops_based_on_time(&mut self) {
        self.rain_drops.retain(|drop| {
            let elapsed = drop.t.elapsed().as_secs_f32();
            elapsed < self.rain_drop_ttl
        });
    }

    pub(crate) fn wave_to_color(&self, fb: Vec<Vec<f32>>) -> Vec<Vec<(u8, u8, u8)>> {
        let mut colors = vec![vec![(0, 0, 0); self.width]; self.height];
        for i in 0..fb.len() {
            for j in 0..fb[i].len() {
                let a_norm = fb[i][j];
                let desaturation = 0.5;
                let tint = 0.25;
                let (r, g, b) = water_rgb(a_norm, desaturation, tint);
                colors[i][j] = (r, g, b);
            }
        }
        colors
    }

    pub(crate) fn generate_wave(&self, fb: &mut Vec<Vec<f32>>) {
        // let mut y = -3.14;

        let h = self.height as f32;
        let w = self.width as f32;

        let dx = 6.28 / w;
        let dy = 6.28 / h;
        for rain_drop in self.rain_drops.iter() {
            let tx = rain_drop.x;
            let ty = rain_drop.y;

            let y_top = -0.5 + ty as f32 / h;
            let x_left = -0.5 + tx as f32 / w;

            let t = rain_drop.t.elapsed().as_secs_f32();

            let mut y = -3.14 + y_top * 6.28;
            for yy in 0..self.height {
                let mut x = -3.14 + x_left * 6.28;

                for xx in 0..self.width {
                    // fb[yy][xx] += wave_xy(rain_drop.k, x, y, t);
                    fb[yy][xx] += wave_xy_lut(rain_drop.k, x, y, t);
                    // fb[yy][xx] = 0.0;
                    x += dx;
                }
                y += dy;
            }
        }
    }
}

fn water_rgb(a_norm: f32, desaturation: f32, tint: f32) -> (u8, u8, u8) {
    // Clamp inputs
    let a_norm = a_norm.clamp(-1.0, 1.0);
    let d = desaturation.clamp(0.0, 1.0);
    let t = tint.clamp(0.0, 1.0);

    // Original saturated color
    let (r, g, b) = if a_norm >= 0.0 {
        (
            (0.0 + (180.0 - 0.0) * a_norm),
            (120.0 + (240.0 - 120.0) * a_norm),
            (200.0 + (255.0 - 200.0) * a_norm),
        )
    } else {
        let a_norm = -a_norm;
        (
            (0.0 + (0.0 - 0.0) * a_norm),
            (120.0 - (120.0 - 30.0) * a_norm),
            (200.0 - (200.0 - 80.0) * a_norm),
        )
    };

    // Calculate luminance for desaturation
    let l = 0.299 * r + 0.587 * g + 0.114 * b;

    // Define green tint color
    let green_tint = (80.0, 160.0, 80.0);

    // Blend toward green tint
    let r_tinted = r + t * (green_tint.0 - r);
    let g_tinted = g + t * (green_tint.1 - g);
    let b_tinted = b + t * (green_tint.2 - b);

    // Desaturate by blending toward luminance
    let r_desat = r_tinted + d * (l - r_tinted);
    let g_desat = g_tinted + d * (l - g_tinted);
    let b_desat = b_tinted + d * (l - b_tinted);

    (
        r_desat.round().clamp(0.0, 255.0) as u8,
        g_desat.round().clamp(0.0, 255.0) as u8,
        b_desat.round().clamp(0.0, 255.0) as u8,
    )
}

// compute the wave color
// based on the distance from the center
// and the time
// the wave is a sine wave
// the color is based on the distance from the center
fn wave_xy(k: f32, x: f32, y: f32, t: f32) -> f32 {
    let w = 3.5;
    let d = (x * x + y * y).sqrt();
    // let d = x * x + y * y;
    let wt = w * t;
    // Replace the sine computation with the lookup table
    //let mut wave = math::sin_lut_f32(k * d - wt);
    let mut wave = (k * d - wt).sin();

    //wave *= 0.7;

    wave /= 2.0 + 1.0 * t;

    let dr = 1.2 * w;
    if d > wt || d < (wt - dr) {
        wave = 0.0;
    } else {
        let edge_dist = (wt - d).min(d - (wt - dr));
        let factor = (edge_dist / dr).clamp(0.0, 1.0);
        wave *= factor;
    }

    wave
}

const X_MIN: f32 = -6.3;
const X_MAX: f32 = 6.3;
const Y_MIN: f32 = -6.3;
const Y_MAX: f32 = 6.3;
const T_MAX: f32 = 5.0;
const RESOLUTION: usize = 400; // Number of steps for x, y, and t

lazy_static::lazy_static! {
    static ref LOOKUP_TABLE: std::sync::RwLock<Vec<Vec<Vec<f32>>>> = std::sync::RwLock::new(vec![vec![vec![0.0; RESOLUTION]; RESOLUTION]; RESOLUTION]);
}

pub fn gen_wave_xy_lookup(k: f32) {
    let dx = (X_MAX - X_MIN) / (RESOLUTION as f32);
    let dy = (Y_MAX - Y_MIN) / (RESOLUTION as f32);
    let dt = T_MAX / (RESOLUTION as f32);

    for tx in 0..RESOLUTION {
        for ty in 0..RESOLUTION {
            for tt in 0..RESOLUTION {
                let x_val = X_MIN + tx as f32 * dx;
                let y_val = Y_MIN + ty as f32 * dy;
                let t_val = tt as f32 * dt;
                let d = (x_val * x_val + y_val * y_val).sqrt();
                let wt = 3.5 * t_val;
                let mut wave = (k * d - wt).sin();
                wave *= 0.7;
                wave /= 2.0 + 1.0 * t_val;

                let dr = 1.2 * 3.5;
                if d > wt || d < (wt - dr) {
                    wave = 0.0;
                } else {
                    let edge_dist = (wt - d).min(d - (wt - dr));
                    let factor = (edge_dist / dr).clamp(0.0, 1.0);
                    wave *= factor;
                }

                LOOKUP_TABLE.write().unwrap()[tx][ty][tt] = wave;
            }
        }
    }
}

fn wave_xy_lut(k: f32, x: f32, y: f32, t: f32) -> f32 {
    // Use the lookup table

    let lookup_table = LOOKUP_TABLE.read().unwrap();
    // Use the lookup table
    let dx = (X_MAX - X_MIN) / (RESOLUTION as f32);
    let dy = (Y_MAX - Y_MIN) / (RESOLUTION as f32);
    let dt = T_MAX / (RESOLUTION as f32);

    let tx = ((x - X_MIN) / dx).clamp(0.0, (RESOLUTION - 1) as f32) as usize;
    let ty = ((y - Y_MIN) / dy).clamp(0.0, (RESOLUTION - 1) as f32) as usize;
    let tt = (t / dt).clamp(0.0, (RESOLUTION - 1) as f32) as usize;

    lookup_table[tx][ty][tt]
}
