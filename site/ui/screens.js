export function createScreenController({ landingView, builderView, gameShell, startBtn, bootStatus }) {
  function showLanding() {
    landingView.classList.remove("hidden");
    builderView.classList.add("hidden");
    gameShell.classList.add("hidden");
  }

  function showBuilder() {
    landingView.classList.add("hidden");
    builderView.classList.remove("hidden");
    gameShell.classList.add("hidden");
  }

  function showGame() {
    landingView.classList.add("hidden");
    builderView.classList.add("hidden");
    gameShell.classList.remove("hidden");
  }

  function setBootStatus(text) {
    bootStatus.textContent = text;
  }

  function setStartEnabled(enabled) {
    startBtn.disabled = !enabled;
  }

  function onStart(handler) {
    startBtn.addEventListener("click", handler);
  }

  return {
    showLanding,
    showBuilder,
    showGame,
    setBootStatus,
    setStartEnabled,
    onStart,
  };
}
