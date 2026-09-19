import 'beercss';

export function setupTheme() {
  // This has nothing to do with our brand colors, but it results in a fitting default that matches our color direction.
  ui("theme", "#ff37aa");

  // Automatic switching between light and dark mode without page reload.
  const modeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const updateTheme = () => ui("mode", modeQuery.matches ? "dark" : "light");

  updateTheme();
  modeQuery.addEventListener("change", updateTheme);
}