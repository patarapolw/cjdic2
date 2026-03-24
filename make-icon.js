import sharp from "sharp";
import { spawnSync } from "child_process";

async function resizeAndMakeWhiteTransparent(inputPath, outputPath) {
  const whiteThreshold = 250; // treat >= as white
  const img = sharp(inputPath).ensureAlpha();

  // Get metadata to compute resize dimensions preserving aspect ratio
  const meta = await img.metadata();
  const origW = meta.width || 0;
  const origH = meta.height || 0;
  if (!origW || !origH) throw new Error("Unable to read image dimensions");

  const targetW = Math.max(origW, origH);
  const targetH = targetW;

  // Resize and output raw RGBA buffer
  const { data, info } = await img
    .resize(targetW, targetH, { fit: "fill" })
    .raw()
    .toBuffer({ resolveWithObject: true });

  const { width: w, height: h, channels } = info; // channels should be 4 (RGBA)
  if (channels < 3) throw new Error("Unexpected channel count");

  // Check if image already has any transparency
  let hasTransparency = false;
  for (let i = 0; i < data.length; i += channels) {
    if (data[i + 3] < 255) {
      hasTransparency = true;
      break;
    }
  }

  if (!hasTransparency) {
    // Modify alpha for near-white pixels
    for (let i = 0; i < data.length; i += channels) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = channels === 4 ? data[i + 3] : 255;

      if (
        a > 0 &&
        r >= whiteThreshold &&
        g >= whiteThreshold &&
        b >= whiteThreshold
      ) {
        if (channels === 4) data[i + 3] = 0;
        else {
          // expand to RGBA if input lacked alpha
          // create a new buffer would be required; throw for simplicity
          throw new Error(
            "Input image missing alpha channel; ensure ensureAlpha() was applied.",
          );
        }
      }
    }
  }

  // Recreate and save PNG
  await sharp(data, { raw: { width: w, height: h, channels } })
    .png({ compressionLevel: 9 })
    .toFile(outputPath);
}

const [, , input] = process.argv;
if (!input) {
  throw new Error(`Input is required: input=${input}`);
}

// Example usage
resizeAndMakeWhiteTransparent(input, "app-icon.png")
  .then(() => spawnSync("pnpm tauri icon", { shell: true, stdio: "inherit" }))
  .catch((err) => console.error(err));
