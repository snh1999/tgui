import { computed, ref, watch } from "vue";

export type ColorFormat = "hex" | "rgb" | "hsl";

function getRandomColor() {
  return {
    h: Math.random() * 360,
    s: 0.5 + Math.random() * 0.5,
    b: 0.7 + Math.random() * 0.3,
  };
}

export function clamp(v: number, lo = 0, hi = 1) {
  return Math.max(lo, Math.min(hi, v));
}

export function sanitizeHexInput(raw: string): string {
  const body = raw
    .replace(/^#/, "")
    .replace(/[^0-9a-fA-F]/g, "")
    .slice(0, 8);
  if (body.length <= 4) {
    return "#" + body.slice(0, 4);
  }

  if (body.length > 4 && body.length < 6) {
    return "#" + body.slice(0, 6).padEnd(6, "0");
  }
  return "#" + body.slice(0, 8);
}

function hsvToRgb(h: number, s: number, v: number) {
  h /= 360;
  let r = 0,
    g = 0,
    b = 0;
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);
  switch (i % 6) {
    case 0:
      r = v;
      g = t;
      b = p;
      break;
    case 1:
      r = q;
      g = v;
      b = p;
      break;
    case 2:
      r = p;
      g = v;
      b = t;
      break;
    case 3:
      r = p;
      g = q;
      b = v;
      break;
    case 4:
      r = t;
      g = p;
      b = v;
      break;
    case 5:
      r = v;
      g = p;
      b = q;
      break;
  }
  return {
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255),
  };
}

function rgbToHsv(r: number, g: number, b: number) {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b),
    min = Math.min(r, g, b),
    d = max - min;
  let h = 0;
  const s = max === 0 ? 0 : d / max;
  const v = max;
  if (max !== min) {
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h /= 6;
  }
  return { h: h * 360, s, v };
}

function hsvToHsl(h: number, s: number, v: number) {
  const l = v * (1 - s / 2);
  const sl = l === 0 || l === 1 ? 0 : (v - l) / Math.min(l, 1 - l);
  return {
    h: Math.round(h),
    s: Math.round(sl * 100),
    l: Math.round(l * 100),
  };
}

function hslToHsv(h: number, s: number, l: number) {
  const v = l + s * Math.min(l, 1 - l);
  const sv = v === 0 ? 0 : 2 * (1 - l / v);
  return {
    h,
    s: sv,
    v,
  };
}

function rgbToHex(r: number, g: number, b: number) {
  return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
}

function parseHex(
  hex: string
): { r: number; g: number; b: number; a: number } | null {
  let clean = hex.replace("#", "");

  if (clean.length === 3 || clean.length === 4) {
    clean = clean
      .split("")
      .map((c) => c + c)
      .join("");
  }

  if (clean.length === 6) {
    const m = /^([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(clean);
    if (!m) return null;
    return {
      r: parseInt(m[1], 16),
      g: parseInt(m[2], 16),
      b: parseInt(m[3], 16),
      a: 1,
    };
  }
  if (clean.length === 8) {
    const m = /^([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(clean);
    if (!m) return null;
    return {
      r: parseInt(m[1], 16),
      g: parseInt(m[2], 16),
      b: parseInt(m[3], 16),
      a: Math.round((parseInt(m[4], 16) / 255) * 100) / 100,
    };
  }
  return null;
}

function alphaToHex(a: number) {
  return Math.round(a * 255)
    .toString(16)
    .padStart(2, "0");
}

export function useColorPicker() {
  const randomColor = getRandomColor();
  const hue = ref(randomColor.h);
  const saturation = ref(randomColor.s);
  const brightness = ref(randomColor.b);
  const alpha = ref(1);
  const format = ref<ColorFormat>("hex");

  const currentRgb = computed(() =>
    hsvToRgb(hue.value, saturation.value, brightness.value)
  );
  const currentHex = computed(() =>
    rgbToHex(currentRgb.value.r, currentRgb.value.g, currentRgb.value.b)
  );
  const currentHsl = computed(() =>
    hsvToHsl(hue.value, saturation.value, brightness.value)
  );

  // UI helpers
  const hueColor = computed(() => {
    const { r, g, b } = hsvToRgb(hue.value, 1, 1);
    return `rgb(${r},${g},${b})`;
  });

  const currentRgba = computed(
    () =>
      `rgba(${currentRgb.value.r},${currentRgb.value.g},${currentRgb.value.b},${alpha.value})`
  );

  const currentHexWithAlpha = computed(() =>
    alpha.value < 1
      ? currentHex.value + alphaToHex(alpha.value)
      : currentHex.value
  );

  // Display value for copy
  const displayValue = computed(() => {
    const { r, g, b } = currentRgb.value;
    const { h, s, l } = currentHsl.value;
    const a = alpha.value;

    switch (format.value) {
      case "hex":
        return currentHexWithAlpha.value;
      case "rgb":
        return a < 1
          ? `rgba(${r}, ${g}, ${b}, ${a.toFixed(2)})`
          : `rgb(${r}, ${g}, ${b})`;
      case "hsl":
        return a < 1
          ? `hsla(${h}, ${s}%, ${l}%, ${a.toFixed(2)})`
          : `hsl(${h}, ${s}%, ${l}%)`;
    }
  });

  // Input buffers: grouped objects (temporary state)
  const hexInput = ref("");
  const rgbInputs = ref({ r: 0, g: 0, b: 0, a: 100 });
  const hslInputs = ref({ h: 0, s: 0, l: 0, a: 100 });

  // Sync buffers from truth
  watch(
    [hue, saturation, brightness, alpha],
    () => {
      const { r, g, b } = currentRgb.value;
      const { h, s, l } = currentHsl.value;

      hexInput.value = currentHexWithAlpha.value;
      rgbInputs.value = { r, g, b, a: Math.round(alpha.value * 100) };
      hslInputs.value = { h, s, l, a: Math.round(alpha.value * 100) };
    },
    { immediate: true }
  );

  // Apply: Hex → HSV
  function applyFromHex(hex?: string) {
    if (!hex) return;
    const parsed = parseHex(hex);
    if (!parsed) return;
    const hsv = rgbToHsv(parsed.r, parsed.g, parsed.b);
    hue.value = hsv.h;
    saturation.value = hsv.s;
    brightness.value = hsv.v;
    alpha.value = parsed.a;
  }

  function applyHex() {
    const parsed = parseHex(hexInput.value);
    if (!parsed) {
      hexInput.value = currentHexWithAlpha.value;
      return;
    }
    const hsv = rgbToHsv(parsed.r, parsed.g, parsed.b);
    hue.value = hsv.h;
    saturation.value = hsv.s;
    brightness.value = hsv.v;
    alpha.value = parsed.a;
  }

  // Apply: RGB → HSV
  function applyRgb() {
    const r = clamp(rgbInputs.value.r, 0, 255);
    const g = clamp(rgbInputs.value.g, 0, 255);
    const b = clamp(rgbInputs.value.b, 0, 255);
    const a = clamp(rgbInputs.value.a, 0, 100);

    const hsv = rgbToHsv(r, g, b);
    hue.value = hsv.h;
    saturation.value = hsv.s;
    brightness.value = hsv.v;
    alpha.value = a / 100;

    rgbInputs.value = { r, g, b, a };
  }

  // Apply: HSL → HSV (direct, no RGB middleman)
  function applyHsl() {
    const h = clamp(hslInputs.value.h, 0, 360);
    const s = clamp(hslInputs.value.s, 0, 100) / 100;
    const l = clamp(hslInputs.value.l, 0, 100) / 100;
    const a = clamp(hslInputs.value.a, 0, 100);

    const hsv = hslToHsv(h, s, l);
    hue.value = hsv.h;
    saturation.value = hsv.s;
    brightness.value = hsv.v;
    alpha.value = a / 100;

    hslInputs.value = { h, s, l: Math.round(l * 100), a };
  }

  function applyPreset(hex: string) {
    applyFromHex(hex);
    alpha.value = 1;
  }

  return {
    hue,
    saturation,
    brightness,
    alpha,
    format,
    currentHex,
    currentHexWithAlpha,
    hueColor,
    currentRgba,
    displayValue,
    hexInput,
    rgbInputs,
    hslInputs,
    applyFromHex,
    applyHex,
    applyRgb,
    applyHsl,
    applyPreset,
  };
}
