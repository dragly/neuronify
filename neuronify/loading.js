export default function () {
  const progress = document.getElementById("progress");
  return {
    onProgress({ current, total }) {
      progress.style.transform = `scaleX(${current / total})`;
    },
  };
}
