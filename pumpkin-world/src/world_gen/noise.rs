use pumpkin_core::random::Random;

pub fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

pub fn lerp2(delta_x: f64, delta_y: f64, x0y0: f64, x1y0: f64, x0y1: f64, x1y1: f64) -> f64 {
    lerp(
        delta_y,
        lerp(delta_x, x0y0, x1y0),
        lerp(delta_x, x0y1, x1y1),
    )
}

pub fn lerp3(
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    x0y0z0: f64,
    x1y0z0: f64,
    x0y1z0: f64,
    x1y1z0: f64,
    x0y0z1: f64,
    x1y0z1: f64,
    x0y1z1: f64,
    x1y1z1: f64,
) -> f64 {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, x0y0z0, x1y0z0, x0y1z0, x1y1z0),
        lerp2(delta_x, delta_y, x0y0z1, x1y0z1, x0y1z1, x1y1z1),
    )
}

struct Gradient {
    x: i32,
    y: i32,
    z: i32,
}

pub struct SimplexNoiseSampler {
    permutation: Box<[u8]>,
    x_origin: f64,
    y_origin: f64,
    z_origin: f64,
}

impl SimplexNoiseSampler {
    const GRADIENTS: [Gradient; 16] = [
        Gradient { x: 1, y: 1, z: 0 },
        Gradient { x: -1, y: 1, z: 0 },
        Gradient { x: 1, y: -1, z: 0 },
        Gradient { x: -1, y: -1, z: 0 },
        Gradient { x: 1, y: 0, z: 1 },
        Gradient { x: -1, y: 0, z: 1 },
        Gradient { x: 1, y: 0, z: -1 },
        Gradient { x: -1, y: 0, z: -1 },
        Gradient { x: 0, y: 1, z: 1 },
        Gradient { x: 0, y: -1, z: 1 },
        Gradient { x: 0, y: 1, z: -1 },
        Gradient { x: 0, y: -1, z: -1 },
        Gradient { x: 1, y: 1, z: 0 },
        Gradient { x: 0, y: -1, z: 1 },
        Gradient { x: -1, y: 1, z: 0 },
        Gradient { x: 0, y: -1, z: -1 },
    ];

    const SQRT_3: f64 = 1.732050807568877293527446341505872367f64;
    const SKEW_FACTOR_2D: f64 = 0.5f64 * (Self::SQRT_3 - 1f64);
    const UNSKEW_FACTOR_2D: f64 = (3f64 - Self::SQRT_3) / 6f64;

    pub fn new(random: &mut impl Random) -> Self {
        let x_origin = random.next_f64() * 256f64;
        let y_origin = random.next_f64() * 256f64;
        let z_origin = random.next_f64() * 256f64;

        let mut permutation = [0u8; 256];

        permutation
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i as u8);

        for i in 0..256 {
            let j = random.next_bounded_i32((256 - i) as i32) as usize;
            permutation.swap(i, i + j);
        }

        Self {
            permutation: Box::new(permutation),
            x_origin,
            y_origin,
            z_origin,
        }
    }

    fn map(&self, input: i32) -> i32 {
        self.permutation[(input & 0xFF) as usize] as i32
    }

    fn dot(gradient: &Gradient, x: f64, y: f64, z: f64) -> f64 {
        gradient.x as f64 * x + gradient.y as f64 * y + gradient.z as f64 * z
    }

    fn grad(gradient_index: usize, x: f64, y: f64, z: f64, distance: f64) -> f64 {
        let d = distance - x * x - y * y - z * z;
        if d < 0f64 {
            0f64
        } else {
            let d = d * d;
            d * d * Self::dot(&Self::GRADIENTS[gradient_index], x, y, z)
        }
    }

    pub fn sample_2d(&self, x: f64, y: f64) -> f64 {
        let d = (x + y) * Self::SKEW_FACTOR_2D;
        let i = (x + d).floor() as i32;
        let j = (y + d).floor() as i32;

        let e = (i.wrapping_add(j)) as f64 * Self::UNSKEW_FACTOR_2D;
        let f = i as f64 - e;
        let g = j as f64 - e;

        let h = x - f;
        let k = y - g;

        let (l, m) = if h > k { (1, 0) } else { (0, 1) };

        let n = h - l as f64 + Self::UNSKEW_FACTOR_2D;
        let o = k - m as f64 + Self::UNSKEW_FACTOR_2D;
        let p = h - 1f64 + 2f64 * Self::UNSKEW_FACTOR_2D;
        let q = k - 1f64 + 2f64 * Self::UNSKEW_FACTOR_2D;

        let r = i & 0xFF;
        let s = j & 0xFF;

        let t = self.map(r + self.map(s)) % 12;
        let u = self.map(r.wrapping_add(l).wrapping_add(self.map(s.wrapping_add(m)))) % 12;
        let v = self.map(r.wrapping_add(1).wrapping_add(self.map(s.wrapping_add(1)))) % 12;

        let w = Self::grad(t as usize, h, k, 0f64, 0.5f64);
        let z = Self::grad(u as usize, n, o, 0f64, 0.5f64);
        let aa = Self::grad(v as usize, p, q, 0f64, 0.5f64);

        70f64 * (w + z + aa)
    }

    pub fn sample_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let e = (x + y + z) * 0.3333333333333333f64;

        let i = (x + e).floor() as i32;
        let j = (y + e).floor() as i32;
        let k = (z + e).floor() as i32;

        let g = (i.wrapping_add(j).wrapping_add(k)) as f64 * 0.16666666666666666f64;
        let h = i as f64 - g;
        let l = j as f64 - g;
        let m = k as f64 - g;

        let n = x - h;
        let o = y - l;
        let p = z - m;

        let (q, r, s, t, u, v) = if n >= o {
            if o >= p {
                (1, 0, 0, 1, 1, 0)
            } else if n >= p {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if o < p {
            (0, 0, 1, 0, 1, 1)
        } else if n < p {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        let w = n - q as f64 + 0.16666666666666666f64;
        let aa = o - r as f64 + 0.16666666666666666f64;
        let ab = p - s as f64 + 0.16666666666666666f64;

        let ac = n - t as f64 + 0.3333333333333333f64;
        let ad = o - u as f64 + 0.3333333333333333f64;
        let ae = p - v as f64 + 0.3333333333333333f64;

        let af = n - 1f64 + 0.5f64;
        let ag = o - 1f64 + 0.5f64;
        let ah = p - 1f64 + 0.5f64;

        let ai = i & 0xFF;
        let aj = j & 0xFF;
        let ak = k & 0xFF;

        let al = self.map(ai.wrapping_add(self.map(aj.wrapping_add(self.map(ak))))) % 12;
        let am = self.map(
            ai.wrapping_add(q).wrapping_add(
                self.map(
                    aj.wrapping_add(r)
                        .wrapping_add(self.map(ak.wrapping_add(s))),
                ),
            ),
        ) % 12;
        let an = self.map(
            ai.wrapping_add(t).wrapping_add(
                self.map(
                    aj.wrapping_add(u)
                        .wrapping_add(self.map(ak.wrapping_add(v))),
                ),
            ),
        ) % 12;
        let ao = self.map(
            ai.wrapping_add(1).wrapping_add(
                self.map(
                    aj.wrapping_add(1)
                        .wrapping_add(self.map(ak.wrapping_add(1))),
                ),
            ),
        ) % 12;

        let ap = Self::grad(al as usize, n, o, p, 0.6f64);
        let aq = Self::grad(am as usize, w, aa, ab, 0.6f64);
        let ar = Self::grad(an as usize, ac, ad, ae, 0.6f64);
        let az = Self::grad(ao as usize, af, ag, ah, 0.6f64);

        32f64 * (ap + aq + ar + az)
    }
}

pub struct PerlinNoiseSampler {
    permutation: Box<[u8]>,
    x_origin: f64,
    y_origin: f64,
    z_origin: f64,
}

impl PerlinNoiseSampler {
    pub fn new(random: &mut impl Random) -> Self {
        let x_origin = random.next_f64() * 256f64;
        let y_origin = random.next_f64() * 256f64;
        let z_origin = random.next_f64() * 256f64;

        let mut permutation = [0u8; 256];

        permutation
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = i as u8);

        for i in 0..256 {
            let j = random.next_bounded_i32((256 - i) as i32) as usize;
            permutation.swap(i, i + j);
        }

        Self {
            permutation: Box::new(permutation),
            x_origin,
            y_origin,
            z_origin,
        }
    }

    pub fn sample_flat_y(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample_no_fade(x, y, z, 0f64, 0f64)
    }

    pub fn sample_no_fade(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let trans_x = x + self.x_origin;
        let trans_y = y + self.y_origin;
        let trans_z = z + self.z_origin;

        let x_int = trans_x.floor() as i32;
        let y_int = trans_y.floor() as i32;
        let z_int = trans_z.floor() as i32;

        let x_dec = trans_x - x_int as f64;
        let y_dec = trans_y - y_int as f64;
        let z_dec = trans_z - z_int as f64;

        let y_noise = if y_scale != 0f64 {
            let raw_y_dec = if y_max >= 0f64 && y_max < y_dec {
                y_max
            } else {
                y_dec
            };
            (raw_y_dec / y_scale + 1.0E-7f32 as f64).floor() * y_scale
        } else {
            0f64
        };

        self.sample(x_int, y_int, z_int, x_dec, y_dec - y_noise, z_dec, y_dec)
    }

    fn grad(hash: i32, x: f64, y: f64, z: f64) -> f64 {
        SimplexNoiseSampler::dot(
            &SimplexNoiseSampler::GRADIENTS[(hash & 15) as usize],
            x,
            y,
            z,
        )
    }

    fn perlin_fade(value: f64) -> f64 {
        value * value * value * (value * (value * 6f64 - 15f64) + 10f64)
    }

    fn map(&self, input: i32) -> i32 {
        (self.permutation[(input & 0xFF) as usize] & 0xFF) as i32
    }

    #[allow(clippy::too_many_arguments)]
    fn sample(
        &self,
        x: i32,
        y: i32,
        z: i32,
        local_x: f64,
        local_y: f64,
        local_z: f64,
        fade_local_y: f64,
    ) -> f64 {
        let i = self.map(x);
        let j = self.map(x.wrapping_add(1));
        let k = self.map(i.wrapping_add(y));

        let l = self.map(i.wrapping_add(y).wrapping_add(1));
        let m = self.map(j.wrapping_add(y));
        let n = self.map(j.wrapping_add(y).wrapping_add(1));

        let d = Self::grad(self.map(k.wrapping_add(z)), local_x, local_y, local_z);
        let e = Self::grad(
            self.map(m.wrapping_add(z)),
            local_x - 1f64,
            local_y,
            local_z,
        );
        let f = Self::grad(
            self.map(l.wrapping_add(z)),
            local_x,
            local_y - 1f64,
            local_z,
        );
        let g = Self::grad(
            self.map(n.wrapping_add(z)),
            local_x - 1f64,
            local_y - 1f64,
            local_z,
        );
        let h = Self::grad(
            self.map(k.wrapping_add(z).wrapping_add(1)),
            local_x,
            local_y,
            local_z - 1f64,
        );
        let o = Self::grad(
            self.map(m.wrapping_add(z).wrapping_add(1)),
            local_x - 1f64,
            local_y,
            local_z - 1f64,
        );
        let p = Self::grad(
            self.map(l.wrapping_add(z).wrapping_add(1)),
            local_x,
            local_y - 1f64,
            local_z - 1f64,
        );
        let q = Self::grad(
            self.map(n.wrapping_add(z).wrapping_add(1)),
            local_x - 1f64,
            local_y - 1f64,
            local_z - 1f64,
        );
        let r = Self::perlin_fade(local_x);
        let s = Self::perlin_fade(fade_local_y);
        let t = Self::perlin_fade(local_z);

        lerp3(r, s, t, d, e, f, g, h, o, p, q)
    }
}

struct OctavePerlinNoiseSampler {
    octave_samplers: Box<[SimplexNoiseSampler]>,
    persistence: f64,
    lacunarity: f64,
}

impl OctavePerlinNoiseSampler {
    pub fn new(random: &mut impl Random, octaves: &[i32]) -> Self {
        let mut octaves = Vec::from_iter(octaves);
        octaves.sort();

        let i = -**octaves.first().expect("Should have some octaves");
        let j = **octaves.last().expect("Should have some octaves");
        let k = i.wrapping_add(j).wrapping_add(1);

        let sampler = SimplexNoiseSampler::new(random);
        let l = j;
        let mut samplers: Vec<SimplexNoiseSampler> = vec![];

        if j >= 0 && j < k && octaves.contains(&&0) {
            samplers[0] = sampler;
        }

        for m in (j + 1)..k {
            if m >= 0 && octaves.contains(&&(l - m)) {
                samplers[m as usize] = SimplexNoiseSampler::new(random);
            } else {
                random.skip(262);
            }
        }

        if j > 0 {
            let n = (sampler.sample_3d(sampler.x_origin, sampler.y_origin, sampler.z_origin)
                * 9.223372E18f32 as f64) as i64;
        }
    }
}

#[cfg(test)]
mod simplex_noise_sampler_test {
    use std::ops::Deref;

    use pumpkin_core::random::{xoroshiro128::Xoroshiro, Random};

    use crate::world_gen::noise::SimplexNoiseSampler;

    #[test]
    fn test_create() {
        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);
        let sampler = SimplexNoiseSampler::new(&mut rand);
        assert_eq!(sampler.x_origin, 48.58072036717974f64);
        assert_eq!(sampler.y_origin, 110.73235882678037f64);
        assert_eq!(sampler.z_origin, 65.26438852860176f64);

        let permutation: [u8; 256] = [
            159, 113, 41, 143, 203, 123, 95, 177, 25, 79, 229, 219, 194, 60, 130, 14, 83, 99, 24,
            202, 207, 232, 167, 152, 220, 201, 29, 235, 87, 147, 74, 160, 155, 97, 111, 31, 85,
            205, 115, 50, 13, 171, 77, 237, 149, 116, 209, 174, 169, 109, 221, 9, 166, 84, 54, 216,
            121, 106, 211, 16, 69, 244, 65, 192, 183, 146, 124, 37, 56, 45, 193, 158, 126, 217, 36,
            255, 162, 163, 230, 103, 63, 90, 191, 214, 20, 138, 32, 39, 238, 67, 64, 105, 250, 140,
            148, 114, 68, 75, 200, 161, 239, 125, 227, 199, 101, 61, 175, 107, 129, 240, 170, 51,
            139, 86, 186, 145, 212, 178, 30, 251, 89, 226, 120, 153, 47, 141, 233, 2, 179, 236, 1,
            19, 98, 21, 164, 108, 11, 23, 91, 204, 119, 88, 165, 195, 168, 26, 48, 206, 128, 6, 52,
            118, 110, 180, 197, 231, 117, 7, 3, 135, 224, 58, 82, 78, 4, 59, 222, 18, 72, 57, 150,
            43, 246, 100, 122, 112, 53, 133, 93, 17, 27, 210, 142, 234, 245, 80, 22, 46, 185, 172,
            71, 248, 33, 173, 76, 35, 40, 92, 228, 127, 254, 70, 42, 208, 73, 104, 187, 62, 154,
            243, 189, 241, 34, 66, 249, 94, 8, 12, 134, 132, 102, 242, 196, 218, 181, 28, 38, 15,
            151, 157, 247, 223, 198, 55, 188, 96, 0, 182, 49, 190, 156, 10, 215, 252, 131, 137,
            184, 176, 136, 81, 44, 213, 253, 144, 225, 5,
        ];
        assert_eq!(sampler.permutation.deref(), permutation);
    }

    #[test]
    fn test_sample_2d() {
        let data1 = [
            ((-50000, 0), -0.013008608535752102),
            ((-49999, 1000), 0.0),
            ((-49998, 2000), -0.03787856584046271),
            ((-49997, 3000), 0.0),
            ((-49996, 4000), 0.5015373706471664),
            ((-49995, 5000), -0.032797908620906514),
            ((-49994, 6000), -0.19158655563621785),
            ((-49993, 7000), 0.49893473629544977),
            ((-49992, 8000), 0.31585737840402556),
            ((-49991, 9000), 0.43909577227435836),
        ];

        let data2 = [
            (
                (-3.134738528791615E8, 5.676610095659718E7),
                0.018940199193618792,
            ),
            (
                (-1369026.560586418, 3.957311252810864E8),
                -0.1417598930091471,
            ),
            (
                (6.439373693833767E8, -3.36218773041759E8),
                0.07129176668335062,
            ),
            (
                (1.353820060118252E8, -3.204701624793043E8),
                0.330648835988156,
            ),
            (
                (-6906850.625560562, 1.0153663948838013E8),
                0.46826928755778685,
            ),
            (
                (-7.108376621385525E7, -2.029413580824217E8),
                -0.515950097501492,
            ),
            (
                (1.0591429119126628E8, -4.7911044364543396E8),
                -0.5467822192664874,
            ),
            (
                (4.04615501401398E7, -3.074409286586152E8),
                0.7470460844090322,
            ),
            (
                (-4.8645283544246924E8, -3.922570151180015E8),
                0.8521699147242563,
            ),
            (
                (2.861710031285905E8, -1.8973201372718483E8),
                0.1889297962671115,
            ),
            (
                (2.885407603819252E8, -3.358708100884505E7),
                0.24006029504945695,
            ),
            (
                (3.6548491156354237E8, 7.995429702025633E7),
                -0.8114171447379924,
            ),
            (
                (1.3298684552869435E8, 3.6743804723880893E8),
                0.07042306408164949,
            ),
            (
                (-1.3123184148036437E8, -2.722300890805201E8),
                0.5093850689193259,
            ),
            (
                (-5.56047682304707E8, 3.554803693060646E8),
                -0.6343788467687929,
            ),
            (
                (5.638216625134594E8, -2.236907346192737E8),
                0.5848746152449286,
            ),
            (
                (-5.436956979127073E7, -1.129261611506945E8),
                -0.05456282199582522,
            ),
            (
                (1.0915760091641709E8, 1.932642099859593E7),
                -0.273739377096594,
            ),
            (
                (-6.73911758014991E8, -2.2147483413687566E8),
                0.05464681163741797,
            ),
            (
                (-2.4827386778136212E8, -2.6640208832089204E8),
                -0.0902449424742273,
            ),
        ];

        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);

        let sampler = SimplexNoiseSampler::new(&mut rand);
        for ((x, y), sample) in data1 {
            assert_eq!(sampler.sample_2d(x as f64, y as f64), sample);
        }

        for ((x, y), sample) in data2 {
            assert_eq!(sampler.sample_2d(x, y), sample);
        }
    }

    #[test]
    fn test_sample_3d() {
        let data = [
            (
                (
                    -3.134738528791615E8,
                    5.676610095659718E7,
                    2.011711832498507E8,
                ),
                -0.07626353895981935,
            ),
            (
                (-1369026.560586418, 3.957311252810864E8, 6.797037355570006E8),
                0.0,
            ),
            (
                (
                    6.439373693833767E8,
                    -3.36218773041759E8,
                    -3.265494249695775E8,
                ),
                -0.5919400355725402,
            ),
            (
                (
                    1.353820060118252E8,
                    -3.204701624793043E8,
                    -4.612474746056331E8,
                ),
                -0.5220477236433517,
            ),
            (
                (
                    -6906850.625560562,
                    1.0153663948838013E8,
                    2.4923185478305575E8,
                ),
                -0.39146687767898636,
            ),
            (
                (
                    -7.108376621385525E7,
                    -2.029413580824217E8,
                    2.5164602748045415E8,
                ),
                -0.629386846329711,
            ),
            (
                (
                    1.0591429119126628E8,
                    -4.7911044364543396E8,
                    -2918719.2277242197,
                ),
                0.5427502531663232,
            ),
            (
                (
                    4.04615501401398E7,
                    -3.074409286586152E8,
                    5.089118769334092E7,
                ),
                -0.4273080639878097,
            ),
            (
                (
                    -4.8645283544246924E8,
                    -3.922570151180015E8,
                    2.3741632952563038E8,
                ),
                0.32129944093252394,
            ),
            (
                (
                    2.861710031285905E8,
                    -1.8973201372718483E8,
                    -3.2653143323982143E8,
                ),
                0.35839032946039706,
            ),
            (
                (
                    2.885407603819252E8,
                    -3.358708100884505E7,
                    -1.4480399660676318E8,
                ),
                -0.02451312935907038,
            ),
            (
                (
                    3.6548491156354237E8,
                    7.995429702025633E7,
                    2.509991661702412E8,
                ),
                -0.36830526266318003,
            ),
            (
                (
                    1.3298684552869435E8,
                    3.6743804723880893E8,
                    5.791092458225288E7,
                ),
                -0.023683302916542803,
            ),
            (
                (
                    -1.3123184148036437E8,
                    -2.722300890805201E8,
                    2.1601883778132245E7,
                ),
                -0.261629562325043,
            ),
            (
                (
                    -5.56047682304707E8,
                    3.554803693060646E8,
                    3.1647392358159083E8,
                ),
                -0.4959372930161496,
            ),
            (
                (
                    5.638216625134594E8,
                    -2.236907346192737E8,
                    -5.0562852022285646E8,
                ),
                -0.06079315675880484,
            ),
            (
                (
                    -5.436956979127073E7,
                    -1.129261611506945E8,
                    -1.7909512156895646E8,
                ),
                -0.37726907424345196,
            ),
            (
                (
                    1.0915760091641709E8,
                    1.932642099859593E7,
                    -3.405060533753616E8,
                ),
                0.37747828159811136,
            ),
            (
                (
                    -6.73911758014991E8,
                    -2.2147483413687566E8,
                    -4.531457195005102E7,
                ),
                -0.32929020207000603,
            ),
            (
                (
                    -2.4827386778136212E8,
                    -2.6640208832089204E8,
                    -3.354675096522197E8,
                ),
                -0.3046390200444667,
            ),
        ];

        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);

        let sampler = SimplexNoiseSampler::new(&mut rand);
        for ((x, y, z), sample) in data {
            assert_eq!(sampler.sample_3d(x, y, z), sample);
        }
    }
}

#[cfg(test)]
mod perlin_noise_sampler_test {
    use std::ops::Deref;

    use pumpkin_core::random::{xoroshiro128::Xoroshiro, Random};

    use crate::world_gen::noise::PerlinNoiseSampler;

    #[test]
    fn test_create() {
        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);

        let sampler = PerlinNoiseSampler::new(&mut rand);
        assert_eq!(sampler.x_origin, 48.58072036717974);
        assert_eq!(sampler.y_origin, 110.73235882678037);
        assert_eq!(sampler.z_origin, 65.26438852860176);

        let permutation: [u8; 256] = [
            159, 113, 41, 143, 203, 123, 95, 177, 25, 79, 229, 219, 194, 60, 130, 14, 83, 99, 24,
            202, 207, 232, 167, 152, 220, 201, 29, 235, 87, 147, 74, 160, 155, 97, 111, 31, 85,
            205, 115, 50, 13, 171, 77, 237, 149, 116, 209, 174, 169, 109, 221, 9, 166, 84, 54, 216,
            121, 106, 211, 16, 69, 244, 65, 192, 183, 146, 124, 37, 56, 45, 193, 158, 126, 217, 36,
            255, 162, 163, 230, 103, 63, 90, 191, 214, 20, 138, 32, 39, 238, 67, 64, 105, 250, 140,
            148, 114, 68, 75, 200, 161, 239, 125, 227, 199, 101, 61, 175, 107, 129, 240, 170, 51,
            139, 86, 186, 145, 212, 178, 30, 251, 89, 226, 120, 153, 47, 141, 233, 2, 179, 236, 1,
            19, 98, 21, 164, 108, 11, 23, 91, 204, 119, 88, 165, 195, 168, 26, 48, 206, 128, 6, 52,
            118, 110, 180, 197, 231, 117, 7, 3, 135, 224, 58, 82, 78, 4, 59, 222, 18, 72, 57, 150,
            43, 246, 100, 122, 112, 53, 133, 93, 17, 27, 210, 142, 234, 245, 80, 22, 46, 185, 172,
            71, 248, 33, 173, 76, 35, 40, 92, 228, 127, 254, 70, 42, 208, 73, 104, 187, 62, 154,
            243, 189, 241, 34, 66, 249, 94, 8, 12, 134, 132, 102, 242, 196, 218, 181, 28, 38, 15,
            151, 157, 247, 223, 198, 55, 188, 96, 0, 182, 49, 190, 156, 10, 215, 252, 131, 137,
            184, 176, 136, 81, 44, 213, 253, 144, 225, 5,
        ];
        assert_eq!(sampler.permutation.deref(), permutation);
    }

    #[test]
    fn test_no_y() {
        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);
        let sampler = PerlinNoiseSampler::new(&mut rand);

        let values = [
            (
                (
                    -3.134738528791615E8,
                    5.676610095659718E7,
                    2.011711832498507E8,
                ),
                0.38582139614602945,
            ),
            (
                (-1369026.560586418, 3.957311252810864E8, 6.797037355570006E8),
                0.15777501333157193,
            ),
            (
                (
                    6.439373693833767E8,
                    -3.36218773041759E8,
                    -3.265494249695775E8,
                ),
                -0.2806135912409497,
            ),
            (
                (
                    1.353820060118252E8,
                    -3.204701624793043E8,
                    -4.612474746056331E8,
                ),
                -0.15052865500837787,
            ),
            (
                (
                    -6906850.625560562,
                    1.0153663948838013E8,
                    2.4923185478305575E8,
                ),
                -0.3079300694558318,
            ),
            (
                (
                    -7.108376621385525E7,
                    -2.029413580824217E8,
                    2.5164602748045415E8,
                ),
                0.03051312670440398,
            ),
            (
                (
                    1.0591429119126628E8,
                    -4.7911044364543396E8,
                    -2918719.2277242197,
                ),
                -0.11775123159138573,
            ),
            (
                (
                    4.04615501401398E7,
                    -3.074409286586152E8,
                    5.089118769334092E7,
                ),
                0.08763639340713025,
            ),
            (
                (
                    -4.8645283544246924E8,
                    -3.922570151180015E8,
                    2.3741632952563038E8,
                ),
                0.08857245482456311,
            ),
            (
                (
                    2.861710031285905E8,
                    -1.8973201372718483E8,
                    -3.2653143323982143E8,
                ),
                -0.2378339698793312,
            ),
            (
                (
                    2.885407603819252E8,
                    -3.358708100884505E7,
                    -1.4480399660676318E8,
                ),
                -0.46661747461279457,
            ),
            (
                (
                    3.6548491156354237E8,
                    7.995429702025633E7,
                    2.509991661702412E8,
                ),
                0.1671543972176835,
            ),
            (
                (
                    1.3298684552869435E8,
                    3.6743804723880893E8,
                    5.791092458225288E7,
                ),
                -0.2704070746642889,
            ),
            (
                (
                    -1.3123184148036437E8,
                    -2.722300890805201E8,
                    2.1601883778132245E7,
                ),
                0.05049887915906969,
            ),
            (
                (
                    -5.56047682304707E8,
                    3.554803693060646E8,
                    3.1647392358159083E8,
                ),
                -0.21178547899422662,
            ),
            (
                (
                    5.638216625134594E8,
                    -2.236907346192737E8,
                    -5.0562852022285646E8,
                ),
                0.03351245780858128,
            ),
            (
                (
                    -5.436956979127073E7,
                    -1.129261611506945E8,
                    -1.7909512156895646E8,
                ),
                0.31670010349494726,
            ),
            (
                (
                    1.0915760091641709E8,
                    1.932642099859593E7,
                    -3.405060533753616E8,
                ),
                -0.13987439655026918,
            ),
            (
                (
                    -6.73911758014991E8,
                    -2.2147483413687566E8,
                    -4.531457195005102E7,
                ),
                0.07824440437151846,
            ),
            (
                (
                    -2.4827386778136212E8,
                    -2.6640208832089204E8,
                    -3.354675096522197E8,
                ),
                -0.2989735599541437,
            ),
        ];

        for ((x, y, z), sample) in values {
            assert_eq!(sampler.sample_flat_y(x, y, z), sample);
        }
    }

    #[test]
    fn test_no_fade() {
        let mut rand = Xoroshiro::from_seed(111);
        assert_eq!(rand.next_i32(), -1467508761);
        let sampler = PerlinNoiseSampler::new(&mut rand);

        let values = [
            (
                (
                    -3.134738528791615E8,
                    5.676610095659718E7,
                    2.011711832498507E8,
                    -1369026.560586418,
                    3.957311252810864E8,
                ),
                23234.47859421248,
            ),
            (
                (
                    6.797037355570006E8,
                    6.439373693833767E8,
                    -3.36218773041759E8,
                    -3.265494249695775E8,
                    1.353820060118252E8,
                ),
                -0.016403984198221984,
            ),
            (
                (
                    -3.204701624793043E8,
                    -4.612474746056331E8,
                    -6906850.625560562,
                    1.0153663948838013E8,
                    2.4923185478305575E8,
                ),
                0.3444286491766397,
            ),
            (
                (
                    -7.108376621385525E7,
                    -2.029413580824217E8,
                    2.5164602748045415E8,
                    1.0591429119126628E8,
                    -4.7911044364543396E8,
                ),
                0.03051312670440398,
            ),
            (
                (
                    -2918719.2277242197,
                    4.04615501401398E7,
                    -3.074409286586152E8,
                    5.089118769334092E7,
                    -4.8645283544246924E8,
                ),
                0.3434020232968479,
            ),
            (
                (
                    -3.922570151180015E8,
                    2.3741632952563038E8,
                    2.861710031285905E8,
                    -1.8973201372718483E8,
                    -3.2653143323982143E8,
                ),
                -0.07935517045771859,
            ),
            (
                (
                    2.885407603819252E8,
                    -3.358708100884505E7,
                    -1.4480399660676318E8,
                    3.6548491156354237E8,
                    7.995429702025633E7,
                ),
                -0.46661747461279457,
            ),
            (
                (
                    2.509991661702412E8,
                    1.3298684552869435E8,
                    3.6743804723880893E8,
                    5.791092458225288E7,
                    -1.3123184148036437E8,
                ),
                0.0723439870279631,
            ),
            (
                (
                    -2.722300890805201E8,
                    2.1601883778132245E7,
                    -5.56047682304707E8,
                    3.554803693060646E8,
                    3.1647392358159083E8,
                ),
                -0.656560662515624,
            ),
            (
                (
                    5.638216625134594E8,
                    -2.236907346192737E8,
                    -5.0562852022285646E8,
                    -5.436956979127073E7,
                    -1.129261611506945E8,
                ),
                0.03351245780858128,
            ),
            (
                (
                    -1.7909512156895646E8,
                    1.0915760091641709E8,
                    1.932642099859593E7,
                    -3.405060533753616E8,
                    -6.73911758014991E8,
                ),
                -0.2089142558681482,
            ),
            (
                (
                    -2.2147483413687566E8,
                    -4.531457195005102E7,
                    -2.4827386778136212E8,
                    -2.6640208832089204E8,
                    -3.354675096522197E8,
                ),
                0.38250837565598395,
            ),
            (
                (
                    3.618095500266467E8,
                    -1.785261966631494E8,
                    8.855575989580283E7,
                    -1.3702508894700047E8,
                    -3.564818414428105E8,
                ),
                0.00883370523171791,
            ),
            (
                (
                    3.585592594479808E7,
                    1.8822208340571395E8,
                    -386327.524558296,
                    -2.613548000006699E8,
                    1995562.4304017993,
                ),
                -0.27653878487738676,
            ),
            (
                (
                    3.0800276873619422E7,
                    1.166750302259058E7,
                    8.502636255675305E7,
                    4.347409652503064E8,
                    1.0678086363325526E8,
                ),
                -0.13800758751097497,
            ),
            (
                (
                    -2.797805968820768E8,
                    9.446376468140173E7,
                    2.2821543438325477E8,
                    -4.8176550369786626E8,
                    7.316871126959312E7,
                ),
                0.05505478945301634,
            ),
            (
                (
                    -2.236596113898912E7,
                    1.5296478602495643E8,
                    3.903966235164034E8,
                    9.40479475527148E7,
                    1.0948229366673347E8,
                ),
                0.1158678618158655,
            ),
            (
                (
                    3.5342596632385695E8,
                    3.1584773170834744E8,
                    -2.1860087172846535E8,
                    -1.8126626716239208E8,
                    -2.5263456116162892E7,
                ),
                -0.354953975313882,
            ),
            (
                (
                    -1.2711958434031656E8,
                    -4.541988855460623E7,
                    -1.375878074907788E8,
                    6.72693784001799E7,
                    6815739.665531283,
                ),
                -0.23849179316215247,
            ),
            (
                (
                    1.2660906027019228E8,
                    -3.3769609799741164E7,
                    -3.4331505330046E8,
                    -6.663866659430536E7,
                    -1.6603843763414428E8,
                ),
                0.07974650858448407,
            ),
        ];

        for ((x, y, z, y_scale, y_max), sample) in values {
            assert_eq!(sampler.sample_no_fade(x, y, z, y_scale, y_max), sample);
        }
    }
}
