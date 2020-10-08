// https://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/

fn rgb_to_hsl(rgb_color: [f32; 3]) -> [f32; 3] {
  let [r, g, b] = rgb_color;
  let min = r.min(g).min(b);
  let max = r.max(g).max(b);
  let l = (min + max) / 2.0;
  let s = if l <= 0.5 {
      (max - min) / (max + min)
  } else {
      (max - min) / (2.0 - max - min)
  };
  let h = (if min == max {
      0.0
  } else if r == max {
      (g - b) / (max - min)
  } else if g == max {
      2.0 + (b - r) / (max - min)
  } else {
      4.0 + (r - g) / (max - min)
  }) / 6.0;
  [h, s, l]
}

fn hsl_to_rgb(hsl_color: [f32; 3]) -> [f32; 3] {
  let [h, s, l] = hsl_color;
  let t1 = if l < 0.5 {
      l * (1.0 + s)
  } else {
      l + s - l * s
  };
  let t2 = 2.0 * l - t1;
  let t_r = h + 1.0 / 3.0;
  let t_g = h;
  let t_b = h - 1.0 / 3.0;
  let r = if 6.0 * t_r < 1.0 {
      t2 + (t1 - t2) * 6.0 * t_r
  } else if 2.0 * t_r < 1.0 {
      t1
  } else if 1.5 * t_r < 1.0 {
      t2 + (t1 - t2) * (2.0 / 3.0 - t_r) * 6.0
  } else {
      t2
  };
  let g = if 6.0 * t_g < 1.0 {
      t2 + (t1 - t2) * 6.0 * t_g
  } else if 2.0 * t_g < 1.0 {
      t1
  } else if 1.5 * t_g < 1.0 {
      t2 + (t1 - t2) * (2.0 / 3.0 - t_g) * 6.0
  } else {
      t2
  };
  let b = if 6.0 * t_b < 1.0 {
      t2 + (t1 - t2) * 6.0 * t_b
  } else if 2.0 * t_b < 1.0 {
      t1
  } else if 1.5 * t_b < 1.0 {
      t2 + (t1 - t2) * (2.0 / 3.0 - t_b) * 6.0
  } else {
      t2
  };
  [r, g, b]
}

fn contrast(hsl_color_a: [f32; 3], hsl_color_b: [f32; 3]) -> f32 {
  let [_, _, l_a] = hsl_color_a;
  let [_, _, l_b] = hsl_color_b;
  let l1 = l_a.max(l_b);
  let l2 = l_a.min(l_b);
  (l1 + 0.05) / (l2 + 0.05)
}