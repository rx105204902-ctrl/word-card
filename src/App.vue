<script setup>
import { onBeforeUnmount, onMounted, ref } from "vue";
import {
  cursorPosition,
  currentMonitor,
  getCurrentWindow,
  primaryMonitor,
} from "@tauri-apps/api/window";
import { LogicalSize, PhysicalPosition } from "@tauri-apps/api/dpi";

const isCompact = ref(true);
const isSettings = ref(false);
const settingsSection = ref("word-bank");
const tooltip = ref({
  visible: false,
  text: "",
  x: 0,
  y: 0,
  position: "bottom",
});
let desiredCompact = true;
let resizeInFlight = false;

const FULL_SIZE = { width: 350, height: 150 };
const COMPACT_SIZE = { width: 150, height: 50 };
const CURSOR_POLL_INTERVAL_MS = 120;
const SNAP_THRESHOLD = 16;
const SNAP_DEBOUNCE_MS = 120;

let cachedWindow = null;
let isRepositioning = false;
let unlistenMove = null;
let unlistenFocus = null;
let cursorPollTimer = null;
let snapAnchor = null;
let snapInFlight = false;
let snapDebounceTimer = null;

const getAppWindow = () => {
  if (!cachedWindow) {
    try {
      cachedWindow = getCurrentWindow();
    } catch (error) {
      console.warn("Failed to access Tauri window APIs", error);
      return null;
    }
  }
  return cachedWindow;
};

const clamp = (value, min, max) => Math.min(Math.max(value, min), max);

const setWindowSize = async (width, height) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.setSize(new LogicalSize(width, height));
  } catch (error) {
    console.warn("Failed to resize window", error);
  }
};

const setWindowPosition = async (position) => {
  const appWindow = getAppWindow();
  if (!appWindow || !position) {
    return;
  }
  try {
    isRepositioning = true;
    await appWindow.setPosition(position);
  } catch (error) {
    console.warn("Failed to reposition window", error);
  } finally {
    isRepositioning = false;
  }
};

const getWorkAreaRect = async () => {
  const monitor = (await currentMonitor()) ?? (await primaryMonitor());
  if (!monitor) {
    return null;
  }
  const area = monitor.workArea ?? monitor;
  const areaPosition = area.position ?? monitor.position;
  const areaSize = area.size ?? monitor.size;
  if (!areaPosition || !areaSize) {
    return null;
  }
  const left = areaPosition.x;
  const top = areaPosition.y;
  const right = areaPosition.x + areaSize.width;
  const bottom = areaPosition.y + areaSize.height;
  return {
    left,
    top,
    right,
    bottom,
    width: areaSize.width,
    height: areaSize.height,
  };
};

const getWindowRect = async () => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return null;
  }
  try {
    const [position, size] = await Promise.all([
      appWindow.outerPosition(),
      appWindow.outerSize(),
    ]);
    const left = position.x;
    const top = position.y;
    const right = position.x + size.width;
    const bottom = position.y + size.height;
    return {
      left,
      top,
      right,
      bottom,
      width: size.width,
      height: size.height,
      centerX: left + size.width / 2,
      centerY: top + size.height / 2,
    };
  } catch (error) {
    console.warn("Failed to read window position", error);
    return null;
  }
};

const getPhysicalSizeFromLogical = async (size) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return null;
  }
  try {
    const scaleFactor = await appWindow.scaleFactor();
    return new LogicalSize(size.width, size.height).toPhysical(scaleFactor);
  } catch (error) {
    console.warn("Failed to resolve physical size", error);
    return null;
  }
};

const getPhysicalLengthFromLogical = async (length) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return null;
  }
  try {
    const scaleFactor = await appWindow.scaleFactor();
    const physicalSize = new LogicalSize(length, length).toPhysical(scaleFactor);
    return Math.max(1, Math.round(physicalSize.width));
  } catch (error) {
    console.warn("Failed to resolve physical length", error);
    return null;
  }
};

const getSnapBounds = (area, size) => {
  const minX = area.left;
  const minY = area.top;
  const maxX = Math.max(minX, area.right - size.width);
  const maxY = Math.max(minY, area.bottom - size.height);
  return { minX, minY, maxX, maxY };
};

const updateSnapAnchor = (area, rect) => {
  if (!area || !rect) {
    return null;
  }
  const kind = resolveCornerKind(area, rect);
  const nextAnchor = buildAnchorFromRect(kind, rect);
  if (nextAnchor) {
    snapAnchor = nextAnchor;
  }
  return nextAnchor;
};

const getSnapTargetFromRect = (rect, area, threshold) => {
  const size = { width: rect.width, height: rect.height };
  const { minX, minY, maxX, maxY } = getSnapBounds(area, size);
  const distLeft = rect.left - area.left;
  const distRight = area.right - rect.right;
  const distTop = rect.top - area.top;
  const distBottom = area.bottom - rect.bottom;
  const nearLeft = Math.abs(distLeft) <= threshold;
  const nearRight = Math.abs(distRight) <= threshold;
  const nearTop = Math.abs(distTop) <= threshold;
  const nearBottom = Math.abs(distBottom) <= threshold;
  if (!nearLeft && !nearRight && !nearTop && !nearBottom) {
    return null;
  }
  let x = rect.left;
  let y = rect.top;
  if (nearLeft && nearTop) {
    x = minX;
    y = minY;
  } else if (nearRight && nearTop) {
    x = maxX;
    y = minY;
  } else if (nearLeft && nearBottom) {
    x = minX;
    y = maxY;
  } else if (nearRight && nearBottom) {
    x = maxX;
    y = maxY;
  } else {
    if (nearLeft) {
      x = minX;
    } else if (nearRight) {
      x = maxX;
    }
    if (nearTop) {
      y = minY;
    } else if (nearBottom) {
      y = maxY;
    }
  }
  const clampedX = clamp(x, minX, maxX);
  const clampedY = clamp(y, minY, maxY);
  return { x: Math.round(clampedX), y: Math.round(clampedY) };
};

const resolveCornerKind = (area, rect) => {
  const centerX = area.left + area.width / 2;
  const centerY = area.top + area.height / 2;
  const isLeft = rect.centerX <= centerX;
  const isTop = rect.centerY <= centerY;
  if (isTop) {
    return isLeft ? "top-left" : "top-right";
  }
  return isLeft ? "bottom-left" : "bottom-right";
};

const buildAnchorFromRect = (kind, rect) => {
  switch (kind) {
    case "top-right":
      return { kind, point: { x: rect.right, y: rect.top } };
    case "bottom-left":
      return { kind, point: { x: rect.left, y: rect.bottom } };
    case "bottom-right":
      return { kind, point: { x: rect.right, y: rect.bottom } };
    case "top-left":
    default:
      return { kind: "top-left", point: { x: rect.left, y: rect.top } };
  }
};

const updateSnapAnchorFromWindow = async () => {
  const [area, rect] = await Promise.all([
    getWorkAreaRect(),
    getWindowRect(),
  ]);
  return updateSnapAnchor(area, rect);
};

const resolvePositionForAnchor = (anchor, area, size) => {
  const { minX, minY, maxX, maxY } = getSnapBounds(area, size);
  let x = minX;
  let y = minY;
  switch (anchor.kind) {
    case "top-right":
      x = anchor.point.x - size.width;
      y = anchor.point.y;
      break;
    case "bottom-left":
      x = anchor.point.x;
      y = anchor.point.y - size.height;
      break;
    case "bottom-right":
      x = anchor.point.x - size.width;
      y = anchor.point.y - size.height;
      break;
    case "top-left":
    default:
      x = anchor.point.x;
      y = anchor.point.y;
      break;
  }
  const clampedX = clamp(x, minX, maxX);
  const clampedY = clamp(y, minY, maxY);
  return { x: Math.round(clampedX), y: Math.round(clampedY) };
};

const snapWindowToEdgesIfNeeded = async () => {
  if (snapInFlight || isRepositioning || resizeInFlight) {
    return;
  }
  snapInFlight = true;
  try {
    const [area, rect, threshold] = await Promise.all([
      getWorkAreaRect(),
      getWindowRect(),
      getPhysicalLengthFromLogical(SNAP_THRESHOLD),
    ]);
    if (!area || !rect) {
      return;
    }
    const effectiveThreshold = threshold ?? SNAP_THRESHOLD;
    const target = getSnapTargetFromRect(rect, area, effectiveThreshold);
    if (!target) {
      updateSnapAnchor(area, rect);
      return;
    }
    const deltaX = Math.abs(target.x - rect.left);
    const deltaY = Math.abs(target.y - rect.top);
    if (deltaX <= 1 && deltaY <= 1) {
      updateSnapAnchor(area, rect);
      return;
    }
    await setWindowPosition(new PhysicalPosition(target.x, target.y));
    await updateSnapAnchorFromWindow();
  } catch (error) {
    console.warn("Failed to snap window to screen edge", error);
  } finally {
    snapInFlight = false;
  }
};

const scheduleSnapToEdges = () => {
  if (snapDebounceTimer) {
    window.clearTimeout(snapDebounceTimer);
  }
  snapDebounceTimer = window.setTimeout(() => {
    snapDebounceTimer = null;
    void snapWindowToEdgesIfNeeded();
  }, SNAP_DEBOUNCE_MS);
};

const positionWindowForAnchor = async (nextSize) => {
  const appWindow = getAppWindow();
  if (!appWindow || !snapAnchor) {
    return;
  }
  try {
    const [area, nextPhysicalSize] = await Promise.all([
      getWorkAreaRect(),
      getPhysicalSizeFromLogical(nextSize),
    ]);
    if (!area || !nextPhysicalSize) {
      return;
    }
    const { x, y } = resolvePositionForAnchor(
      snapAnchor,
      area,
      nextPhysicalSize
    );
    await setWindowPosition(new PhysicalPosition(x, y));
  } catch (error) {
    console.warn("Failed to reposition window from anchor", error);
  }
};

const applyDesiredMode = async () => {
  if (resizeInFlight) {
    return;
  }
  resizeInFlight = true;
  while (true) {
    const nextCompact = desiredCompact;
    const nextSize = nextCompact ? COMPACT_SIZE : FULL_SIZE;
    await updateSnapAnchorFromWindow();
    isCompact.value = nextCompact;
    await setWindowSize(nextSize.width, nextSize.height);
    await positionWindowForAnchor(nextSize);
    if (desiredCompact === nextCompact) {
      break;
    }
  }
  resizeInFlight = false;
};

const requestCompactMode = (compact) => {
  if (desiredCompact === compact && !resizeInFlight) {
    return;
  }
  desiredCompact = compact;
  void applyDesiredMode();
};

const setWindowResizable = async (resizable) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.setResizable(resizable);
  } catch (error) {
    console.warn("Failed to update window resizable flag", error);
  }
};

const setWindowMaximizable = async (maximizable) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.setMaximizable(maximizable);
  } catch (error) {
    console.warn("Failed to update window maximizable flag", error);
  }
};

const startWindowDrag = async () => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.startDragging();
  } catch (error) {
    console.warn("Failed to start window drag", error);
  }
};

const handleDragStart = (event) => {
  if (event.buttons !== 1) {
    return;
  }
  event.preventDefault();
  void startWindowDrag();
};

const isCursorInsideWindow = async () => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return null;
  }
  const [cursor, position, size] = await Promise.all([
    cursorPosition(),
    appWindow.outerPosition(),
    appWindow.outerSize(),
  ]);
  const insideX = cursor.x >= position.x && cursor.x <= position.x + size.width;
  const insideY = cursor.y >= position.y && cursor.y <= position.y + size.height;
  return insideX && insideY;
};

const enterCompact = () => {
  requestCompactMode(true);
};

const exitCompact = () => {
  requestCompactMode(false);
};

const openSettings = () => {
  settingsSection.value = "word-bank";
  isSettings.value = true;
};

const closeSettings = () => {
  isSettings.value = false;
};

const setSettingsSection = (section) => {
  settingsSection.value = section;
};

const showTooltip = (event) => {
  const target = event.currentTarget;
  if (!target || !target.dataset) {
    return;
  }
  const text = target.dataset.tooltip;
  if (!text) {
    return;
  }
  const rect = target.getBoundingClientRect();
  const position = target.dataset.tooltipPosition ?? "bottom";
  const next = {
    visible: true,
    text,
    position,
    x: rect.right,
    y: rect.bottom + 6,
  };
  if (position === "right") {
    next.x = rect.right + 6;
    next.y = rect.top + rect.height / 2;
  }
  tooltip.value = next;
};

const hideTooltip = () => {
  if (tooltip.value.visible) {
    tooltip.value = { ...tooltip.value, visible: false };
  }
};

const updateCompactFromCursor = async () => {
  if (resizeInFlight) {
    return;
  }
  try {
    const isInside = await isCursorInsideWindow();
    if (isInside == null) {
      return;
    }
    if (isInside) {
      if (desiredCompact) {
        exitCompact();
      }
      return;
    }
    if (!desiredCompact) {
      enterCompact();
    }
  } catch (error) {
    console.warn("Failed to sync compact mode from cursor position", error);
  }
};

onMounted(async () => {
  void setWindowResizable(false);
  void setWindowMaximizable(false);
  desiredCompact = true;
  await applyDesiredMode();
  const appWindow = getAppWindow();
  if (appWindow) {
    unlistenMove = await appWindow.onMoved(() => {
      if (!isRepositioning) {
        scheduleSnapToEdges();
      }
    });
    unlistenFocus = await appWindow.onFocusChanged(({ payload }) => {
      if (!payload) {
        enterCompact();
      }
    });
  }
  cursorPollTimer = window.setInterval(
    updateCompactFromCursor,
    CURSOR_POLL_INTERVAL_MS
  );
});

onBeforeUnmount(() => {
  if (unlistenMove) {
    unlistenMove();
  }
  if (unlistenFocus) {
    unlistenFocus();
  }
  if (cursorPollTimer) {
    window.clearInterval(cursorPollTimer);
  }
  if (snapDebounceTimer) {
    window.clearTimeout(snapDebounceTimer);
  }
});
</script>

<template>
  <div
    class="app"
    :class="{ 'is-compact': isCompact }"
  >
    <div v-if="isCompact" class="view view-compact">
      <div class="compact-shell" @mousedown="handleDragStart">
        <span class="word word-compact">serendipity</span>
      </div>
    </div>

    <div v-else class="view view-main">
      <main v-if="!isSettings" class="card">
        <div class="top-row" @mousedown="handleDragStart">
          <div class="proficiency-box level-3" aria-label="Proficiency level 3"></div>
          <button
            class="settings-button icon-button"
            type="button"
            aria-label="Settings"
            @click="openSettings"
            @mousedown.stop
            @mouseenter="showTooltip"
            @mouseleave="hideTooltip"
            data-tooltip="Settings"
          >
            <svg
              aria-hidden="true"
              viewBox="0 0 24 24"
              focusable="false"
            >
              <circle cx="12" cy="12" r="3.5" />
              <path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3" />
              <path d="M4.7 4.7l2.1 2.1M17.2 17.2l2.1 2.1" />
              <path d="M19.3 4.7l-2.1 2.1M6.8 17.2l-2.1 2.1" />
            </svg>
          </button>
        </div>

        <div class="word-line">
          <span class="word">serendipity</span>
          <span class="phonetic">/ser-uhn-dip-i-tee/</span>
        </div>

        <p class="word-cn">机缘巧合</p>

        <div class="example-group">
          <p class="example">She found the quiet cafe by sheer serendipity.</p>
          <p class="example-cn">她因机缘巧合找到了那家安静的咖啡馆。</p>
        </div>

        <div class="nav-actions">
          <button class="nav-button" type="button">Previous</button>
          <button class="nav-button" type="button">Next</button>
        </div>
      </main>
      <section v-else class="settings">
        <header class="settings-header" @mousedown="handleDragStart">
          <button
            class="back-button icon-button"
            type="button"
            aria-label="Back"
            @click="closeSettings"
            @mousedown.stop
            @mouseenter="showTooltip"
            @mouseleave="hideTooltip"
            data-tooltip="Back"
          >
            <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
              <path d="M13 6l-6 6 6 6" />
              <path d="M7 12h10" />
            </svg>
          </button>
          <span class="settings-title">Settings</span>
        </header>
        <div class="settings-body">
          <nav class="settings-nav" aria-label="Settings sections">
            <button
              class="settings-nav-item icon-button"
              :class="{ 'is-active': settingsSection === 'word-bank' }"
              type="button"
              @click="setSettingsSection('word-bank')"
              @mousedown.stop
              @mouseenter="showTooltip"
              @mouseleave="hideTooltip"
              aria-label="Word Bank"
              data-tooltip="Word Bank"
              data-tooltip-position="right"
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <path d="M12 6v12" />
                <path d="M4 6h6a2 2 0 0 1 2 2v10H6a2 2 0 0 0-2 2V6z" />
                <path d="M20 6h-6a2 2 0 0 0-2 2v10h6a2 2 0 0 1 2 2V6z" />
              </svg>
            </button>
            <button
              class="settings-nav-item icon-button"
              :class="{ 'is-active': settingsSection === 'fuzzy-words' }"
              type="button"
              @click="setSettingsSection('fuzzy-words')"
              @mousedown.stop
              @mouseenter="showTooltip"
              @mouseleave="hideTooltip"
              aria-label="Fuzzy Words"
              data-tooltip="Fuzzy Words"
              data-tooltip-position="right"
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <path d="M12 3v4M12 17v4M3 12h4M17 12h4" />
                <path d="M6.5 6.5l2.5 2.5M15 15l2.5 2.5" />
                <path d="M17.5 6.5L15 9M9 15l-2.5 2.5" />
              </svg>
            </button>
            <button
              class="settings-nav-item icon-button"
              :class="{ 'is-active': settingsSection === 'study-calendar' }"
              type="button"
              @click="setSettingsSection('study-calendar')"
              @mousedown.stop
              @mouseenter="showTooltip"
              @mouseleave="hideTooltip"
              aria-label="Study Calendar"
              data-tooltip="Study Calendar"
              data-tooltip-position="right"
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <rect x="4" y="6" width="16" height="14" rx="2" />
                <path d="M8 3v4M16 3v4M4 10h16" />
              </svg>
            </button>
            <button
              class="settings-nav-item icon-button"
              :class="{ 'is-active': settingsSection === 'more' }"
              type="button"
              @click="setSettingsSection('more')"
              @mousedown.stop
              @mouseenter="showTooltip"
              @mouseleave="hideTooltip"
              aria-label="More"
              data-tooltip="More"
              data-tooltip-position="right"
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <circle cx="5" cy="12" r="1.8" />
                <circle cx="12" cy="12" r="1.8" />
                <circle cx="19" cy="12" r="1.8" />
              </svg>
            </button>
          </nav>
          <div class="settings-content">
            <p v-if="settingsSection === 'word-bank'" class="settings-placeholder">
              Word Bank settings will appear here.
            </p>
            <p v-else-if="settingsSection === 'fuzzy-words'" class="settings-placeholder">
              Fuzzy Words settings will appear here.
            </p>
            <p v-else-if="settingsSection === 'study-calendar'" class="settings-placeholder">
              Study Calendar settings will appear here.
            </p>
            <p v-else class="settings-placeholder">
              More settings will appear here.
            </p>
          </div>
        </div>
        <div
          v-show="tooltip.visible"
          class="ui-tooltip"
          :class="{ 'is-right': tooltip.position === 'right' }"
          :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
          role="tooltip"
        >
          {{ tooltip.text }}
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.app {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  padding: 8px;
  display: grid;
  grid-template-rows: 1fr;
  background: var(--glass);
  border: 1px solid var(--stroke);
  border-radius: 12px;
  --icon-size: clamp(20px, 7vw, 24px);
  --nav-icon-size: clamp(26px, 10vw, 34px);
  --nav-column: clamp(44px, 16vw, 64px);
  --tooltip-font: clamp(0.42rem, 1.6vw, 0.52rem);
}

.app.is-compact {
  padding: 0;
}

.view {
  display: grid;
  height: 100%;
  min-height: 0;
}

.view-main {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.view-compact {
  height: 100%;
}

.compact-shell {
  height: 100%;
  border-radius: 8px;
  border: none;
  background: transparent;
  display: grid;
  place-items: center;
  padding: 0 8px;
  box-shadow: none;
  cursor: grab;
}

.compact-shell:active {
  cursor: grabbing;
}

.card {
  height: 100%;
  display: grid;
  grid-template-rows: auto auto auto 1fr auto;
  gap: 4px;
  padding: 0;
  border-radius: 0;
  border: none;
  background: transparent;
  box-shadow: none;
}

.top-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: grab;
}

.top-row:active {
  cursor: grabbing;
}

.proficiency-box {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  border: 1px solid rgba(255, 255, 255, 0.5);
  box-shadow: 0 6px 10px -12px var(--shadow);
}

.settings-button {
  width: var(--icon-size);
  height: var(--icon-size);
  padding: 0;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  color: #1f1d1a;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0;
  line-height: 0;
  box-shadow: 0 8px 12px -14px var(--shadow);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  --icon-glyph-size: calc(var(--icon-size) * 0.62);
}

.icon-button {
  position: relative;
  line-height: 0;
  font-size: 0;
}

.icon-button svg {
  display: block;
  width: var(--icon-glyph-size, 15px);
  height: var(--icon-glyph-size, 15px);
  stroke: currentColor;
  fill: none;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.settings-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 14px -14px var(--shadow);
}

.level-1 {
  background: #2ecc71;
}

.level-2 {
  background: #2bc57b;
}

.level-3 {
  background: #27be8c;
}

.level-4 {
  background: #21b0a2;
}

.level-5 {
  background: #1e6fe0;
}

.word-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.word {
  font-family: "Fraunces", serif;
  font-size: 1.05rem;
  letter-spacing: 0.01em;
}

.word-compact {
  font-size: 0.85rem;
  line-height: 1;
  white-space: nowrap;
}

.phonetic {
  font-size: 0.6rem;
  letter-spacing: 0.08em;
  color: var(--muted);
  white-space: nowrap;
}

.word-cn {
  margin: 0;
  font-size: 0.65rem;
  color: #2a2723;
}

.example-group {
  display: grid;
  gap: 2px;
  align-content: start;
}

.example {
  margin: 0;
  font-size: 0.62rem;
  line-height: 1.3;
  color: #2a2723;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.example-cn {
  margin: 0;
  font-size: 0.58rem;
  line-height: 1.3;
  color: var(--muted);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.nav-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

.nav-button {
  padding: 4px 6px;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
  font-size: 0.48rem;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  cursor: pointer;
  color: #1f1d1a;
  box-shadow: 0 8px 12px -14px var(--shadow);
  line-height: 1.1;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.nav-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 14px -14px var(--shadow);
}

.settings {
  height: 100%;
  display: grid;
  grid-template-rows: auto 1fr;
  gap: 6px;
  min-height: 0;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: grab;
}

.settings-header:active {
  cursor: grabbing;
}

.settings-title {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.back-button {
  width: var(--icon-size);
  height: var(--icon-size);
  padding: 0;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  color: #1f1d1a;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0;
  line-height: 0;
  box-shadow: 0 8px 12px -14px var(--shadow);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  --icon-glyph-size: calc(var(--icon-size) * 0.62);
}

.back-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 14px -14px var(--shadow);
}

.settings-body {
  display: grid;
  grid-template-columns: var(--nav-column) 1fr;
  gap: 8px;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.settings-nav {
  display: grid;
  align-content: start;
  gap: 6px;
  justify-items: center;
  height: 100%;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  max-height: 100%;
  scrollbar-width: none;
  -ms-overflow-style: none;
  padding-right: 2px;
  padding-bottom: 4px;
  -webkit-overflow-scrolling: touch;
}

.settings-nav::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.settings-nav-item {
  width: var(--nav-icon-size);
  height: var(--nav-icon-size);
  padding: 0;
  border-radius: 8px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.5);
  text-align: center;
  cursor: pointer;
  color: #1f1d1a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0;
  line-height: 0;
  transition: background 0.2s ease, border-color 0.2s ease;
  --icon-glyph-size: calc(var(--nav-icon-size) * 0.6);
}

.settings-nav-item.is-active,
.settings-nav-item:hover {
  background: rgba(255, 255, 255, 0.85);
  border-color: var(--stroke);
}

.settings-content {
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.6);
  padding: 8px;
  height: 100%;
  min-height: 0;
  display: grid;
  overflow: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.settings-content::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.settings-placeholder {
  margin: 0;
  font-size: 0.58rem;
  color: var(--muted);
  line-height: 1.4;
}

.ui-tooltip {
  position: fixed;
  z-index: 9999;
  padding: 2px 6px;
  border-radius: 6px;
  background: #1f1d1a;
  color: #fff;
  font-size: var(--tooltip-font);
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  white-space: nowrap;
  pointer-events: none;
  transform: translate(-100%, 0);
}

.ui-tooltip.is-right {
  transform: translate(0, -50%);
}

@media (prefers-reduced-motion: reduce) {
  .settings-button,
  .nav-button,
  .back-button,
  .settings-nav-item,
  .icon-button {
    transition: none;
  }
}
</style>

<style>
@import url("https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;700&family=Space+Grotesk:wght@400;500;600&display=swap");

:root {
  color-scheme: light;
  font-family: "Space Grotesk", "Segoe UI", sans-serif;
  line-height: 1.5;
  font-weight: 400;
  color: #1f1d1a;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;

  --muted: #6b645d;
  --glass: rgba(255, 255, 255, 0.78);
  --glass-strong: rgba(255, 255, 255, 0.9);
  --stroke: rgba(31, 29, 26, 0.12);
  --shadow: rgba(31, 29, 26, 0.18);
  --accent-teal: #1b9aaa;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
}

body {
  margin: 0;
  background:
    radial-gradient(circle at 14% 12%, rgba(246, 189, 96, 0.22), transparent 55%),
    radial-gradient(circle at 86% 10%, rgba(27, 154, 170, 0.12), transparent 50%),
    linear-gradient(140deg, #f7f1e8 0%, #f0f6f4 100%);
  color: #1f1d1a;
  overflow: hidden;
}

#app {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  padding: 0;
  display: flex;
  align-items: stretch;
  justify-content: stretch;
}
</style>
