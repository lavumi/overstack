export function createHud(elements, options = {}) {
  const CRIT_C = options.critConstant ?? 100;
  let lastEnemyName = "Enemy";
  let onAction = () => {};
  let skillSlotEnabled = [false, false, false, false];

  function clampPct(v) {
    return Math.max(0, Math.min(100, v));
  }

  function setBar(fillEl, current, max) {
    const pct = max > 0 ? clampPct((current / max) * 100) : 0;
    fillEl.style.width = `${pct.toFixed(1)}%`;
  }

  function statusKey(statusType) {
    return String(statusType || "").trim().toLowerCase();
  }

  function renderPillList(container, items, renderItem, emptyLabel, extraClass = "") {
    container.innerHTML = "";
    const frag = document.createDocumentFragment();

    if (!items || items.length === 0) {
      const empty = document.createElement("span");
      empty.className = `${extraClass} empty-pill`.trim();
      empty.textContent = emptyLabel;
      frag.appendChild(empty);
      container.appendChild(frag);
      return;
    }

    for (const item of items) {
      frag.appendChild(renderItem(item));
    }

    container.appendChild(frag);
  }

  function renderStatusList(container, statuses) {
    renderPillList(
      container,
      statuses,
      (status) => {
        const chip = document.createElement("span");
        chip.className = "status-pill";
        chip.dataset.status = statusKey(status.status_type);
        chip.textContent = `${status.status_type} x${status.stacks} · ${Number(status.duration).toFixed(1)}s`;
        return chip;
      },
      "No active statuses",
      "status-pill",
    );
  }

  function renderTraitList(container, traits) {
    renderPillList(
      container,
      traits,
      (traitName) => {
        const chip = document.createElement("span");
        chip.className = "trait-pill";
        chip.textContent = traitName;
        return chip;
      },
      "No active traits",
      "trait-pill",
    );
  }

  function critChancePercent(critRate) {
    const rate = Number(critRate);
    if (!Number.isFinite(rate) || rate <= 0) {
      return 0;
    }
    return (rate / (rate + CRIT_C)) * 100;
  }

  function unitStageState(unit) {
    const hp = Number(unit.hp);
    const maxHp = Number(unit.max_hp);
    const gauge = Number(unit.action_gauge);
    if (hp <= 0) return "down";
    if (gauge >= 100) return "ready";
    if (maxHp > 0 && hp / maxHp <= 0.35) return "critical";
    return "idle";
  }

  function unitStateSummary(unit, traitCount) {
    const statuses = unit.statuses || [];
    if (Number(unit.hp) <= 0) {
      return "Unit collapsed";
    }
    if (Number(unit.action_gauge) >= 100) {
      return "Action gauge primed";
    }
    if (statuses.length > 0) {
      return `${statuses.length} status effect${statuses.length > 1 ? "s" : ""} active`;
    }
    if (traitCount > 0) {
      return `${traitCount} trait${traitCount > 1 ? "s" : ""} influencing combat`;
    }
    return "Stable combat posture";
  }

  function setArenaStatus(text) {
    elements.arenaStatus.firstElementChild.textContent = text;
  }

  function setArenaHint(text) {
    elements.arenaHint.textContent = text;
  }

  function setInputPrompt(text) {
    elements.inputPrompt.textContent = text;
  }

  function setEnemyName(name) {
    lastEnemyName = name || "Enemy";
    elements.enemyName.textContent = lastEnemyName;
    elements.stageEnemyName.textContent = lastEnemyName;
  }

  function setCombatLabels(skillNames) {
    elements.actionBasicBtn.textContent = "Basic Attack";
    for (let i = 0; i < elements.actionSkillButtons.length; i += 1) {
      const name = skillNames[i] || "";
      skillSlotEnabled[i] = Boolean(name);
      elements.actionSkillButtons[i].textContent = name || "Empty";
      elements.actionSkillButtons[i].dataset.inactive = skillSlotEnabled[i] ? "false" : "true";
    }
  }

  function setActionButtonsEnabled(enabled) {
    elements.actionBasicBtn.disabled = !enabled;
    for (let i = 0; i < elements.actionSkillButtons.length; i += 1) {
      const button = elements.actionSkillButtons[i];
      button.disabled = !enabled || !skillSlotEnabled[i];
    }
  }

  function reset() {
    elements.statusNode.textContent = "-";
    elements.statusBattle.textContent = "-";
    elements.playerName.textContent = "Player";
    setEnemyName("Enemy");
    elements.stagePlayerName.textContent = "Player";
    elements.stagePlayerVitals.textContent = "HP - / Gauge -";
    elements.stageEnemyVitals.textContent = "HP - / Gauge -";
    elements.stagePlayerState.textContent = "Awaiting deployment";
    elements.stageEnemyState.textContent = "No encounter loaded";
    elements.stagePlayer.dataset.state = "idle";
    elements.stageEnemy.dataset.state = "idle";
    elements.playerHpText.textContent = "-";
    elements.enemyHpText.textContent = "-";
    elements.playerGaugeText.textContent = "-";
    elements.enemyGaugeText.textContent = "-";
    elements.playerAtkMatk.textContent = "-";
    elements.enemyAtkMatk.textContent = "-";
    elements.playerDef.textContent = "-";
    elements.enemyDef.textContent = "-";
    elements.playerMdef.textContent = "-";
    elements.enemyMdef.textContent = "-";
    elements.playerCrit.textContent = "-";
    elements.enemyIntent.textContent = "-";
    skillSlotEnabled = [false, false, false, false];
    setCombatLabels(["", "", "", ""]);
    renderStatusList(elements.playerStatuses, []);
    renderStatusList(elements.enemyStatuses, []);
    renderTraitList(elements.playerTraits, []);
    renderTraitList(elements.enemyTraits, []);
    setBar(elements.playerHpFill, 0, 1);
    setBar(elements.enemyHpFill, 0, 1);
    setBar(elements.playerGaugeFill, 0, 100);
    setBar(elements.enemyGaugeFill, 0, 100);
    setArenaStatus("Ready");
    setArenaHint("Prepare your opening line.");
    setInputPrompt("");
    setActionButtonsEnabled(false);
  }

  function updateFromSnapshot(snapshot) {
    elements.statusNode.textContent = String(snapshot.node_index);
    elements.statusBattle.textContent = String(snapshot.battle_index);

    elements.playerName.textContent = "Player";
    elements.stagePlayerName.textContent = "Player";
    setEnemyName(lastEnemyName || "Enemy");

    const php = Number(snapshot.player.hp);
    const pmax = Number(snapshot.player.max_hp);
    const ehp = Number(snapshot.enemy.hp);
    const emax = Number(snapshot.enemy.max_hp);
    const pg = Number(snapshot.player.action_gauge);
    const eg = Number(snapshot.enemy.action_gauge);

    elements.playerHpText.textContent = `${php.toFixed(0)} / ${pmax.toFixed(0)}`;
    elements.enemyHpText.textContent = `${ehp.toFixed(0)} / ${emax.toFixed(0)}`;
    elements.playerGaugeText.textContent = pg.toFixed(1);
    elements.enemyGaugeText.textContent = eg.toFixed(1);
    elements.stagePlayerVitals.textContent = `${php.toFixed(0)} HP · Gauge ${pg.toFixed(0)}`;
    elements.stageEnemyVitals.textContent = `${ehp.toFixed(0)} HP · Gauge ${eg.toFixed(0)}`;
    elements.stagePlayerState.textContent = unitStateSummary(snapshot.player, (snapshot.player_traits || []).length);
    elements.stageEnemyState.textContent = unitStateSummary(snapshot.enemy, (snapshot.enemy_traits || []).length);
    elements.stagePlayer.dataset.state = unitStageState(snapshot.player);
    elements.stageEnemy.dataset.state = unitStageState(snapshot.enemy);

    setBar(elements.playerHpFill, php, pmax);
    setBar(elements.enemyHpFill, ehp, emax);
    setBar(elements.playerGaugeFill, pg, 100);
    setBar(elements.enemyGaugeFill, eg, 100);

    elements.playerAtkMatk.textContent = `${snapshot.player.atk} / ${snapshot.player.matk}`;
    elements.enemyAtkMatk.textContent = `${snapshot.enemy.atk} / ${snapshot.enemy.matk}`;
    elements.playerDef.textContent = `${snapshot.player.base_def} -> ${snapshot.player.effective_def}`;
    elements.enemyDef.textContent = `${snapshot.enemy.base_def} -> ${snapshot.enemy.effective_def}`;
    elements.playerMdef.textContent = String(snapshot.player.mdef);
    elements.enemyMdef.textContent = String(snapshot.enemy.mdef);
    elements.playerCrit.textContent = `${critChancePercent(snapshot.player.crit_rate).toFixed(1)}%`;
    elements.enemyIntent.textContent = snapshot.enemy_next_intent || "-";
    renderTraitList(elements.playerTraits, snapshot.player_traits || []);
    renderTraitList(elements.enemyTraits, snapshot.enemy_traits || []);
    renderStatusList(elements.playerStatuses, snapshot.player.statuses);
    renderStatusList(elements.enemyStatuses, snapshot.enemy.statuses);
  }

  elements.actionBasicBtn.addEventListener("click", () => onAction(0));
  elements.actionSkillButtons.forEach((button, idx) => {
    button.addEventListener("click", () => onAction(idx + 1));
  });

  return {
    reset,
    updateFromSnapshot,
    setArenaStatus,
    setArenaHint,
    setInputPrompt,
    setCombatLabels,
    setActionButtonsEnabled,
    setEnemyName,
    onAction(handler) {
      onAction = handler;
    },
  };
}
