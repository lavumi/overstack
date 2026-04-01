export function createBuilderUI(elements, options) {
  const {
    startBuildBudget,
    defaultStats,
    statRanges,
    randomSeed32,
    sampleTraitIds,
  } = options;

  let selectableTraitIds = [];
  let selectableTraitNames = [];
  let selectableTraitCosts = [];
  let selectedBuilderTraitIds = [];
  let builderMode = "random";
  let onConfirm = () => {};
  let onCancel = () => {};

  function toNumber(inputEl, fallback = 0) {
    const v = Number.parseFloat(inputEl.value);
    return Number.isFinite(v) ? v : fallback;
  }

  function readStats() {
    return {
      max_hp: toNumber(elements.statInputs.max_hp, defaultStats.max_hp),
      atk: Math.round(toNumber(elements.statInputs.atk, defaultStats.atk)),
      matk: Math.round(toNumber(elements.statInputs.matk, defaultStats.matk)),
      def: Math.round(toNumber(elements.statInputs.def, defaultStats.def)),
      mdef: Math.round(toNumber(elements.statInputs.mdef, defaultStats.mdef)),
      speed: toNumber(elements.statInputs.speed, defaultStats.speed),
      crit_rate: toNumber(elements.statInputs.crit_rate, defaultStats.crit_rate),
      crit_mult: toNumber(elements.statInputs.crit_mult, defaultStats.crit_mult),
    };
  }

  function writeStats(stats) {
    elements.statInputs.max_hp.value = String(stats.max_hp);
    elements.statInputs.atk.value = String(stats.atk);
    elements.statInputs.matk.value = String(stats.matk);
    elements.statInputs.def.value = String(stats.def);
    elements.statInputs.mdef.value = String(stats.mdef);
    elements.statInputs.speed.value = String(stats.speed.toFixed(2));
    elements.statInputs.crit_rate.value = String(stats.crit_rate.toFixed(1));
    elements.statInputs.crit_mult.value = String(stats.crit_mult.toFixed(2));
    refreshBudgetText(stats);
  }

  function calcTotalCost(stats) {
    const hp_cost = Math.max(0, (stats.max_hp - 100) / 5);
    const atk_cost = stats.atk * 1.0;
    const matk_cost = stats.matk * 1.0;
    const def_cost = Math.max(0, stats.def) * 0.8;
    const mdef_cost = stats.mdef * 0.8;
    const speed_cost = Math.max(0, stats.speed - 1.0) * 60;
    const crit_rate_cost = stats.crit_rate / 10;
    const crit_mult_cost = Math.max(0, stats.crit_mult - 1.5) * 40;
    return hp_cost + atk_cost + matk_cost + def_cost + mdef_cost + speed_cost + crit_rate_cost + crit_mult_cost;
  }

  function selectedBuilderTraitName() {
    const selectedId = selectedBuilderTraitIds[0];
    if (!selectedId) {
      return "";
    }
    const idx = selectableTraitIds.indexOf(selectedId);
    return idx >= 0 ? selectableTraitNames[idx] || selectedId : selectedId;
  }

  function selectedBuilderTraitCost() {
    if (!selectedBuilderTraitIds.length) {
      return 0;
    }
    const idx = selectableTraitIds.indexOf(selectedBuilderTraitIds[0]);
    return idx >= 0 ? Number(selectableTraitCosts[idx] || 0) : 0;
  }

  function calcBuildCost(stats = readStats()) {
    const statsCost = calcTotalCost(stats);
    const traitsCost = selectedBuilderTraitCost();
    const totalCost = statsCost + traitsCost;
    const remainingBudget = startBuildBudget - totalCost;
    return {
      statsCost,
      traitsCost,
      totalCost,
      remainingBudget,
    };
  }

  function refreshBudgetText(stats = readStats()) {
    const summary = calcBuildCost(stats);
    elements.builderStatsCostText.textContent = summary.statsCost.toFixed(1);
    elements.builderTraitCostText.textContent = summary.traitsCost.toFixed(0);
    elements.builderBudgetText.textContent = `${summary.totalCost.toFixed(1)} / ${startBuildBudget.toFixed(1)}`;
    elements.builderRemainingText.textContent = summary.remainingBudget.toFixed(1);
    const traitName = selectedBuilderTraitName();
    elements.builderTraitHint.textContent = traitName
      ? `Selected trait: ${traitName}`
      : builderMode === "manual"
        ? "Select one starting trait that fits the remaining budget."
        : "Random mode rolls one weighted trait first, then fills stats with the remaining budget.";
  }

  function setBuilderMode(mode) {
    builderMode = mode;
    const editable = mode === "manual";
    Object.values(elements.statInputs).forEach((el) => {
      el.readOnly = !editable;
    });
  }

  function generateRandomStats(budgetCap = startBuildBudget) {
    const stats = { ...defaultStats };
    let guard = 0;
    while (calcTotalCost(stats) < budgetCap && guard < 2000) {
      guard += 1;
      const roll = Math.random();
      const candidate = { ...stats };

      if (roll < 0.2) candidate.max_hp += 5;
      else if (roll < 0.4) candidate.atk += 1;
      else if (roll < 0.6) candidate.matk += 1;
      else if (roll < 0.72) candidate.def += 1;
      else if (roll < 0.84) candidate.mdef += 1;
      else if (roll < 0.92) candidate.crit_rate += 5;
      else if (roll < 0.97) candidate.speed += 0.05;
      else candidate.crit_mult += 0.05;

      if (candidate.max_hp > statRanges.max_hp[1]) continue;
      if (candidate.atk > statRanges.atk[1]) continue;
      if (candidate.matk > statRanges.matk[1]) continue;
      if (candidate.def > statRanges.def[1]) continue;
      if (candidate.mdef > statRanges.mdef[1]) continue;
      if (candidate.speed > statRanges.speed[1]) continue;
      if (candidate.crit_rate > statRanges.crit_rate[1]) continue;
      if (candidate.crit_mult > statRanges.crit_mult[1]) continue;

      if (calcTotalCost(candidate) <= budgetCap) {
        Object.assign(stats, candidate);
      }
    }

    stats.speed = Number(stats.speed.toFixed(2));
    stats.crit_rate = Number(stats.crit_rate.toFixed(1));
    stats.crit_mult = Number(stats.crit_mult.toFixed(2));
    return stats;
  }

  function renderBuilderTraits() {
    elements.builderTraitChoices.innerHTML = "";
    const frag = document.createDocumentFragment();
    const selectedSet = new Set(selectedBuilderTraitIds);

    for (let i = 0; i < selectableTraitIds.length; i += 1) {
      const id = selectableTraitIds[i];
      const name = selectableTraitNames[i] || id;
      const cost = Number(selectableTraitCosts[i] || 0);
      const button = document.createElement("button");
      button.type = "button";
      button.className = "builder-trait-option";
      if (selectedSet.has(id)) {
        button.classList.add("is-selected");
      }
      button.disabled = builderMode !== "manual";
      button.innerHTML = `
        <span class="builder-trait-name">${name}</span>
        <span class="builder-trait-cost">Cost ${cost}</span>
        <span class="builder-trait-id">${id}</span>
      `;
      button.addEventListener("click", () => {
        if (builderMode !== "manual") return;
        const candidate = [id];
        const stats = readStats();
        const totalCost = calcTotalCost(stats) + Number(cost || 0);
        if (totalCost > startBuildBudget) {
          elements.builderError.textContent = `Trait budget exceeded: ${totalCost.toFixed(1)} / ${startBuildBudget.toFixed(1)}`;
          return;
        }
        selectedBuilderTraitIds = candidate;
        elements.builderError.textContent = "";
        renderBuilderTraits();
        refreshBudgetText();
      });
      frag.appendChild(button);
    }

    elements.builderTraitChoices.appendChild(frag);
  }

  function sampleBuilderTrait() {
    const sampled = sampleTraitIds(randomSeed32(), 1);
    selectedBuilderTraitIds = sampled.slice(0, 1);
  }

  function generateRandomBuild() {
    sampleBuilderTrait();
    const stats = generateRandomStats(startBuildBudget - selectedBuilderTraitCost());
    writeStats(stats);
    renderBuilderTraits();
    refreshBudgetText(stats);
  }

  function validateStatsForConfirm(stats) {
    const errors = [];
    for (const [key, [min, max]] of Object.entries(statRanges)) {
      const value = stats[key];
      if (!Number.isFinite(value) || value < min || value > max) {
        errors.push(`${key} must be in ${min}..${max}`);
      }
    }

    const summary = calcBuildCost(stats);
    if (selectedBuilderTraitIds.length !== 1) {
      errors.push("Select exactly one starting trait");
    }
    if (summary.totalCost > startBuildBudget) {
      errors.push(`Budget exceeded: ${summary.totalCost.toFixed(1)} / ${startBuildBudget.toFixed(1)}`);
    }

    return { ok: errors.length === 0, errors, ...summary };
  }

  function open({ ids, names, costs }) {
    elements.builderError.textContent = "";
    selectableTraitIds = ids;
    selectableTraitNames = names;
    selectableTraitCosts = costs;
    elements.builderModeRandom.checked = true;
    elements.builderModeManual.checked = false;
    setBuilderMode("random");
    generateRandomBuild();
  }

  function reset() {
    selectableTraitIds = [];
    selectableTraitNames = [];
    selectableTraitCosts = [];
    selectedBuilderTraitIds = [];
    elements.builderError.textContent = "";
  }

  elements.builderModeRandom.addEventListener("change", () => {
    if (!elements.builderModeRandom.checked) return;
    setBuilderMode("random");
    generateRandomBuild();
    elements.builderError.textContent = "";
  });

  elements.builderModeManual.addEventListener("change", () => {
    if (!elements.builderModeManual.checked) return;
    setBuilderMode("manual");
    if (selectedBuilderTraitIds.length === 0 && selectableTraitIds[0]) {
      selectedBuilderTraitIds = [selectableTraitIds[0]];
    }
    renderBuilderTraits();
    elements.builderError.textContent = "";
    refreshBudgetText();
  });

  elements.builderRandomBtn.addEventListener("click", () => {
    generateRandomBuild();
    elements.builderError.textContent = "";
  });

  elements.builderRerollBtn.addEventListener("click", () => {
    if (builderMode === "random") {
      generateRandomBuild();
    } else {
      renderBuilderTraits();
      refreshBudgetText();
    }
    elements.builderError.textContent = "";
  });

  Object.values(elements.statInputs).forEach((el) => {
    el.addEventListener("input", () => {
      renderBuilderTraits();
      refreshBudgetText();
    });
  });

  elements.builderConfirmBtn.addEventListener("click", () => {
    const stats = readStats();
    const checked = validateStatsForConfirm(stats);
    elements.builderStatsCostText.textContent = checked.statsCost.toFixed(1);
    elements.builderTraitCostText.textContent = checked.traitsCost.toFixed(0);
    elements.builderBudgetText.textContent = `${checked.totalCost.toFixed(1)} / ${startBuildBudget.toFixed(1)}`;
    elements.builderRemainingText.textContent = checked.remainingBudget.toFixed(1);

    if (!checked.ok) {
      elements.builderError.textContent = checked.errors.join(" | ");
      return;
    }

    elements.builderError.textContent = "";
    onConfirm({ stats, traitIds: [...selectedBuilderTraitIds] });
  });

  elements.builderCancelBtn.addEventListener("click", () => {
    onCancel();
  });

  return {
    open,
    reset,
    onConfirm(handler) {
      onConfirm = handler;
    },
    onCancel(handler) {
      onCancel = handler;
    },
  };
}
