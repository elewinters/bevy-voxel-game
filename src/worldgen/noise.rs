use fastnoise_lite::FastNoiseLite;

/* ------------------ */
/*      constants     */
/* ------------------ */
// #tag constants

pub const FREQUENCY: f32 = 0.006; // essentially the scale of the noise function, lower values zoom in while higher values zoom out

const PLAINS_HEIGHT_VARIATION: f32 = 10.0;
const PLAINS_VALLEY_THRESHOLD: f32 = 0.0; // below this value we'll have valleys
const PLAINS_VALLEY_SMOOTHNESS: f32 = 1.5; // how smooth valleys are
const PLAINS_VALLEY_STEP: f32 = 3.0; // how much smoother the valleys should get the lower they are

const HILLS_HEIGHT_VARIATION: f32 = 50.0;
const HILLS_WAVELENGTH: f32 = 5.0;

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

pub fn generate_noise(perlin_noise: &FastNoiseLite, x: f32, z: f32) -> f32 {
    let plains = perlin_noise.get_noise_2d(x, z);
    let hills = perlin_noise.get_noise_2d(x / HILLS_WAVELENGTH, z / HILLS_WAVELENGTH) * HILLS_HEIGHT_VARIATION;

    // plains valleys
    let plains = if plains < PLAINS_VALLEY_THRESHOLD {
        // we multiply VALLEY_SMOOTHNESS by (VALLEY_STEP.powf(y) so that the deeper the valley the smoother it is
        plains * PLAINS_HEIGHT_VARIATION / (PLAINS_VALLEY_SMOOTHNESS * (PLAINS_VALLEY_STEP.powf(plains.abs())))
    }
    // normal plains
    else {
        (plains * PLAINS_HEIGHT_VARIATION).powf(1.1)
    };

    (plains + hills).floor()
}