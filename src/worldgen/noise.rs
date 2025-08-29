use fastnoise_lite::FastNoiseLite;

/* ------------------ */
/*      constants     */
/* ------------------ */
// #tag constants

pub const FREQUENCY: f32 = 0.006; // essentially the scale of the noise function, lower values zoom in while higher values zoom out
const HEIGHT_VARIATION: f32 = 10.0;

const VALLEY_THRESHOLD: f32 = 0.0; // below this value we'll have valleys
const VALLEY_SMOOTHNESS: f32 = 1.5; // how smooth valleys are
const VALLEY_STEP: f32 = 3.0; // how much smoother the valleys should get the lower they are

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

pub fn generate_noise(perlin_noise: &FastNoiseLite, x: f32, z: f32) -> f32 {
    let plains = perlin_noise.get_noise_2d(x, z);
    let hills = perlin_noise.get_noise_2d(x / 5.0, z / 5.0) * 50.0;

    // valley
    let plains = if plains < VALLEY_THRESHOLD {
        // we multiply VALLEY_SMOOTHNESS by (VALLEY_STEP.powf(y) so that the deeper the valley the smoother it is
        plains * HEIGHT_VARIATION / (VALLEY_SMOOTHNESS * (VALLEY_STEP.powf(plains.abs())))
    }
    // normal terrain
    else {
        (plains * HEIGHT_VARIATION).powf(1.1)
    };

    (plains + hills).floor()
}
