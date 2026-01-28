<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  cursorPosition,
  currentMonitor,
  getCurrentWindow,
  primaryMonitor,
} from "@tauri-apps/api/window";
import { LogicalSize, PhysicalPosition } from "@tauri-apps/api/dpi";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const isCompact = ref(true);
const isSettings = ref(false);
const settingsSection = ref("word-bank");
const hideMode = ref("compact");
const edgeSide = ref("right");
const edgeInsets = ref({ left: 0, right: 0, top: 0, bottom: 0 });
const edgeInsetFallback = ref(0);
const fullWidth = ref(350);
const fullWidthDraft = ref(350);
const wordListCards = ref([]);
const wordBankNotice = ref("");
const wordBankLoading = ref(false);
const dictionaryDownloadNotice = ref("");
const dictionaryDownloadStates = ref({});
const fuzzyWords = ref([]);
const fuzzyWordsNotice = ref("");
const fuzzyWordsLoading = ref(false);
const fuzzyWordSort = ref("marked");
const fuzzySelectedIds = ref([]);
const fuzzyWordDetailId = ref(null);
const fuzzyAudioPlaying = ref(null);
const learningNotice = ref("");
const learningBusy = ref(false);
const currentWord = ref(null);
const remainingWords = ref([]);
const historyStack = ref([]);
const prefetchWords = ref([]);
const prefetchInFlight = ref(false);
const studyCalendarView = ref("calendar");
const studyCalendarCounts = ref([]);
const studyCalendarLoading = ref(false);
const studyCalendarNotice = ref("");
const calendarAnchor = ref(new Date());
const studyCalendarCache = { loadedAt: 0, data: [] };
const studyChartHover = ref({
  visible: false,
  x: 0,
  y: 0,
  value: 0,
});
const selectableWordListCards = computed(() =>
  wordListCards.value.filter((item) => !item.is_system)
);
const hasWordLists = computed(() => selectableWordListCards.value.length > 0);
const hasActiveWordList = computed(() =>
  wordListCards.value.some((item) => item.is_active)
);
const showEmptyState = computed(
  () => !wordBankLoading.value && !hasActiveWordList.value
);
const sortedWordListCards = computed(() => {
  const list = [...wordListCards.value];
  list.sort((a, b) => Number(b.is_active) - Number(a.is_active));
  return list;
});
const downloadedDictionaryNames = computed(
  () => new Set(wordListCards.value.map((item) => item.name))
);
const isDictionaryDownloaded = (name) =>
  downloadedDictionaryNames.value.has(name);
const getDictionaryState = (id) =>
  dictionaryDownloadStates.value[id] ?? {
    status: "idle",
    progress: 0,
    error: "",
    hasTotal: false,
  };
const isDictionaryBusy = (id) => {
  const status = getDictionaryState(id).status;
  return status === "downloading" || status === "importing";
};
const hasCurrentWord = computed(() => Boolean(currentWord.value));
const proficiencyLevel = computed(() => {
  const score = currentWord.value?.proficiency_score ?? 0;
  if (score <= 1) {
    return 1;
  }
  if (score <= 3) {
    return 2;
  }
  if (score <= 5) {
    return 3;
  }
  if (score <= 7) {
    return 4;
  }
  return 5;
});
const proficiencyLabel = computed(
  () => `熟练度等级 ${proficiencyLevel.value}`
);
const displayWord = computed(() => currentWord.value?.word ?? "...");
const trayTooltipWord = computed(() => currentWord.value?.word ?? "");
const displayPhonetic = computed(() => currentWord.value?.phonetic ?? "");
const displayMeaning = computed(
  () => currentWord.value?.part_of_speech_and_meanings ?? ""
);
const displayExample = computed(
  () => currentWord.value?.example_sentence ?? ""
);
const displayExampleTranslation = computed(
  () => currentWord.value?.example_translation ?? ""
);
const canGoPrevious = computed(
  () =>
    hasActiveWordList.value &&
    historyStack.value.length > 0 &&
    !learningBusy.value
);
const canGoNext = computed(
  () => hasActiveWordList.value && !learningBusy.value
);
const canMarkFuzzy = computed(
  () => hasActiveWordList.value && hasCurrentWord.value && !learningBusy.value
);
const hasFuzzyWords = computed(() => fuzzyWords.value.length > 0);
const hasFuzzySelection = computed(() => fuzzySelectedIds.value.length > 0);
const isFuzzyAllSelected = computed(
  () =>
    hasFuzzyWords.value && fuzzySelectedIds.value.length === fuzzyWords.value.length
);
const isEdgeHidden = computed(
  () => isCompact.value && hideMode.value === "edge"
);
const fuzzyWordDetail = computed(
  () =>
    fuzzyWords.value.find((item) => item.id === fuzzyWordDetailId.value) ??
    null
);
const nextLabel = computed(() => {
  if (learningBusy.value) {
    return "加载中";
  }
  if (!hasCurrentWord.value) {
    return "开始";
  }
  return "下一个";
});
const tooltip = ref({
  visible: false,
  text: "",
  x: 0,
  y: 0,
  position: "bottom",
});
const dictionaryDownloads = [
  {
    id: "cet4",
    name: "CET4",
    url: "https://raw.githubusercontent.com/rx105204902-ctrl/vocabulary-database/refs/heads/main/CET-4.csv",
  },
  {
    id: "cet6",
    name: "CET6",
    url: "https://raw.githubusercontent.com/rx105204902-ctrl/vocabulary-database/refs/heads/main/CET-6.csv",
  },
];
let desiredCompact = true;
let resizeInFlight = false;

const BASE_FULL_SIZE = { width: 350, height: 155 };
const APP_PADDING = 8;
const BASE_INNER_SIZE = {
  width: BASE_FULL_SIZE.width - APP_PADDING * 2,
  height: BASE_FULL_SIZE.height - APP_PADDING * 2,
};
const COMPACT_SIZE = { width: 150, height: 50 };
const EDGE_LINE_THICKNESS = 1;
const EDGE_LINE_SIZE = { width: EDGE_LINE_THICKNESS, height: 80 };
const EDGE_FRAME_FALLBACK = 5;
const EDGE_LEFT_NUDGE = 11;
const EDGE_ACTIVATE_PADDING = 8;
const EDGE_REVEAL_HOLD_MS = 500;
const FULL_WIDTH_MIN = BASE_FULL_SIZE.width;
const FULL_WIDTH_MAX = 450;
const FULL_HEIGHT_RATIO = BASE_FULL_SIZE.height / BASE_FULL_SIZE.width;
const CURSOR_POLL_INTERVAL_MS = 120;
const SNAP_THRESHOLD = 16;
const SNAP_DEBOUNCE_MS = 120;
const STUDY_CALENDAR_CACHE_MS = 60 * 1000;
const STUDY_CALENDAR_WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];
const STUDY_CHART_SIZE = { width: 280, height: 140 };
const STUDY_CHART_PADDING = { top: 16, right: 12, bottom: 24, left: 28 };
const STUDY_CHART_HIT_RADIUS = 8;
const STUDY_CALENDAR_MIN_ANCHOR = new Date(2025, 0, 1);

let cachedWindow = null;
let isRepositioning = false;
let unlistenMove = null;
let unlistenFocus = null;
let unlistenHideMode = null;
let cursorPollTimer = null;
let snapAnchor = null;
let snapInFlight = false;
let snapDebounceTimer = null;
let audioPlayer = null;
let edgeRevealUntil = 0;

const fullSize = computed(() => {
  const width = Math.round(
    clamp(fullWidth.value, FULL_WIDTH_MIN, FULL_WIDTH_MAX)
  );
  return {
    width,
    height: Math.round(width * FULL_HEIGHT_RATIO),
  };
});
const fullSizeLabel = computed(
  () => `${fullSize.value.width}px x ${fullSize.value.height}px`
);
const fullSizeDraft = computed(() => {
  const width = Math.round(
    clamp(fullWidthDraft.value, FULL_WIDTH_MIN, FULL_WIDTH_MAX)
  );
  return {
    width,
    height: Math.round(width * FULL_HEIGHT_RATIO),
  };
});
const fullSizeDraftLabel = computed(
  () => `${fullSizeDraft.value.width}px x ${fullSizeDraft.value.height}px`
);
const uiScale = computed(() => {
  const innerWidth = Math.max(1, fullSize.value.width - APP_PADDING * 2);
  const innerHeight = Math.max(1, fullSize.value.height - APP_PADDING * 2);
  const scale = Math.min(
    innerWidth / BASE_INNER_SIZE.width,
    innerHeight / BASE_INNER_SIZE.height
  );
  return clamp(scale, 0.5, 2);
});
const uiScaleStyle = computed(() => ({
  "--ui-scale": uiScale.value,
  "--ui-base-width": `${BASE_INNER_SIZE.width}px`,
  "--ui-base-height": `${BASE_INNER_SIZE.height}px`,
}));

watch(fullWidth, (value) => {
  if (fullWidthDraft.value !== value) {
    fullWidthDraft.value = value;
  }
});
watch(
  trayTooltipWord,
  async (value) => {
    try {
      await invoke("set_tray_tooltip", { word: value });
    } catch (error) {
      console.warn("Failed to update tray tooltip", error);
    }
  },
  { immediate: true }
);
const dailyStudyCountMap = computed(() => {
  const map = {};
  studyCalendarCounts.value.forEach((item) => {
    if (!item?.date) {
      return;
    }
    map[item.date] = Number(item.word_count ?? 0);
  });
  return map;
});
const calendarMinAnchor = computed(() => STUDY_CALENDAR_MIN_ANCHOR.getTime());
const calendarMaxAnchor = computed(() => getCalendarMaxAnchor().getTime());
const canGoPrevMonth = computed(
  () => calendarAnchor.value.getTime() > calendarMinAnchor.value
);
const canGoNextMonth = computed(
  () => calendarAnchor.value.getTime() < calendarMaxAnchor.value
);
const calendarMonthLabel = computed(() => {
  const anchor = calendarAnchor.value;
  const month = String(anchor.getMonth() + 1).padStart(2, "0");
  return `${anchor.getFullYear()}年${month}月`;
});
const calendarCells = computed(() =>
  buildCalendarCells(calendarAnchor.value, dailyStudyCountMap.value)
);
const studyChartSeries = computed(() =>
  buildStudyChartSeries(calendarAnchor.value, dailyStudyCountMap.value)
);
const studyChartYMax = computed(() => {
  const max = Math.max(0, ...studyChartSeries.value.map((item) => item.count));
  if (max <= 0) {
    return 1;
  }
  const step = Math.max(1, Math.ceil(max / 4));
  return step * 4;
});
const studyChartYTicks = computed(() => {
  const height =
    STUDY_CHART_SIZE.height -
    STUDY_CHART_PADDING.top -
    STUDY_CHART_PADDING.bottom;
  const ticks = [];
  const step = studyChartYMax.value / 4;
  for (let i = 0; i <= 4; i += 1) {
    const value = step * i;
    const ratio = value / studyChartYMax.value;
    const y =
      STUDY_CHART_PADDING.top + height - Math.min(1, ratio) * height;
    ticks.push({ value, label: String(value), y });
  }
  return ticks;
});
const studyChartPoints = computed(() => {
  if (!studyChartSeries.value.length) {
    return "";
  }
  const width =
    STUDY_CHART_SIZE.width -
    STUDY_CHART_PADDING.left -
    STUDY_CHART_PADDING.right;
  const height =
    STUDY_CHART_SIZE.height -
    STUDY_CHART_PADDING.top -
    STUDY_CHART_PADDING.bottom;
  const denominator = Math.max(1, studyChartSeries.value.length - 1);
  return studyChartSeries.value
    .map((item, index) => {
      const ratio = item.count / studyChartYMax.value;
      const x =
        STUDY_CHART_PADDING.left + (index / denominator) * width;
      const y =
        STUDY_CHART_PADDING.top + height - Math.min(1, ratio) * height;
      return `${x},${y}`;
    })
    .join(" ");
});
const studyChartPointList = computed(() => {
  const series = studyChartSeries.value;
  if (!series.length) {
    return [];
  }
  const width =
    STUDY_CHART_SIZE.width -
    STUDY_CHART_PADDING.left -
    STUDY_CHART_PADDING.right;
  const height =
    STUDY_CHART_SIZE.height -
    STUDY_CHART_PADDING.top -
    STUDY_CHART_PADDING.bottom;
  const denominator = Math.max(1, series.length - 1);
  return series.map((item, index) => {
    const ratio = item.count / studyChartYMax.value;
    const x = STUDY_CHART_PADDING.left + (index / denominator) * width;
    const y =
      STUDY_CHART_PADDING.top + height - Math.min(1, ratio) * height;
    return { x, y, count: item.count };
  });
});
const studyChartTooltipStyle = computed(() => ({
  left: `${studyChartHover.value.x}px`,
  top: `${studyChartHover.value.y}px`,
}));
const studyChartXAxisLabels = computed(() => {
  const series = studyChartSeries.value;
  if (!series.length) {
    return [];
  }
  const width =
    STUDY_CHART_SIZE.width -
    STUDY_CHART_PADDING.left -
    STUDY_CHART_PADDING.right;
  const denominator = Math.max(1, series.length - 1);
  const indices = [0, Math.floor(series.length / 2), series.length - 1];
  const unique = Array.from(new Set(indices));
  return unique.map((index) => ({
    x: STUDY_CHART_PADDING.left + (index / denominator) * width,
    label: series[index]?.label ?? "",
  }));
});

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

const formatDateKey = (date) => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
};

const setCalendarAnchor = (value) => {
  calendarAnchor.value = clampCalendarAnchor(value);
};

const shiftCalendarMonth = (delta) => {
  const anchor = calendarAnchor.value;
  const next = new Date(anchor.getFullYear(), anchor.getMonth() + delta, 1);
  setCalendarAnchor(next);
};

const distanceToSegment = (x, y, start, end) => {
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  if (dx === 0 && dy === 0) {
    return Math.hypot(x - start.x, y - start.y);
  }
  const t = ((x - start.x) * dx + (y - start.y) * dy) / (dx * dx + dy * dy);
  const clamped = Math.max(0, Math.min(1, t));
  const projX = start.x + clamped * dx;
  const projY = start.y + clamped * dy;
  return Math.hypot(x - projX, y - projY);
};

const handleStudyChartMove = (event) => {
  const points = studyChartPointList.value;
  if (!points.length) {
    studyChartHover.value.visible = false;
    return;
  }
  const rect = event.currentTarget.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  let nearest = null;
  for (let i = 0; i < points.length - 1; i += 1) {
    const start = points[i];
    const end = points[i + 1];
    const distance = distanceToSegment(x, y, start, end);
    if (!nearest || distance < nearest.distance) {
      const midX = (start.x + end.x) / 2;
      const target = x <= midX ? start : end;
      nearest = { ...target, distance };
    }
  }
  if (!nearest && points.length === 1) {
    const only = points[0];
    nearest = {
      ...only,
      distance: Math.hypot(x - only.x, y - only.y),
    };
  }
  if (!nearest || nearest.distance > STUDY_CHART_HIT_RADIUS) {
    studyChartHover.value.visible = false;
    return;
  }
  studyChartHover.value = {
    visible: true,
    x: nearest.x,
    y: Math.max(nearest.y - 18, 6),
    value: nearest.count,
  };
};

const hideStudyChartTooltip = () => {
  studyChartHover.value.visible = false;
};

const normalizeMonthAnchor = (value) =>
  new Date(value.getFullYear(), value.getMonth(), 1);

const getCalendarMaxAnchor = () => {
  const now = new Date();
  return new Date(now.getFullYear(), now.getMonth(), 1);
};

const clampCalendarAnchor = (anchor) => {
  const normalized = normalizeMonthAnchor(anchor);
  const minTime = STUDY_CALENDAR_MIN_ANCHOR.getTime();
  const maxTime = getCalendarMaxAnchor().getTime();
  const value = normalized.getTime();
  if (value < minTime) {
    return new Date(minTime);
  }
  if (value > maxTime) {
    return new Date(maxTime);
  }
  return normalized;
};

const buildCalendarCells = (anchor, countsByDate) => {
  const year = anchor.getFullYear();
  const month = anchor.getMonth();
  const firstDay = new Date(year, month, 1);
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const offset = (firstDay.getDay() + 6) % 7;
  const total = offset + daysInMonth;
  const trailing = (7 - (total % 7)) % 7;
  const totalCells = total + trailing;
  const todayKey = formatDateKey(new Date());
  const cells = [];
  for (let i = 0; i < totalCells; i += 1) {
    const dayIndex = i - offset + 1;
    const date = new Date(year, month, dayIndex);
    const dateKey = formatDateKey(date);
    const isCurrentMonth = date.getMonth() === month;
    const label = isCurrentMonth ? String(date.getDate()) : "";
    const count = isCurrentMonth ? countsByDate[dateKey] ?? 0 : 0;
    cells.push({
      key: `${dateKey}-${i}`,
      label,
      count,
      isCurrentMonth,
      isToday: dateKey === todayKey,
    });
  }
  return cells;
};

const buildStudyChartSeries = (anchor, countsByDate) => {
  const year = anchor.getFullYear();
  const month = anchor.getMonth();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const series = [];
  for (let day = 1; day <= daysInMonth; day += 1) {
    const date = new Date(year, month, day);
    const key = formatDateKey(date);
    series.push({
      key,
      label: String(day),
      count: countsByDate[key] ?? 0,
    });
  }
  return series;
};

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

const setWindowMinSize = async (size) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.setMinSize(size);
  } catch (error) {
    console.warn("Failed to update minimum window size", error);
  }
};

const setWindowMaxSize = async (size) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.setMaxSize(size);
  } catch (error) {
    console.warn("Failed to update maximum window size", error);
  }
};

const setWindowShadow = async (enable) => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    await appWindow.setShadow(enable);
  } catch (error) {
    console.warn("Failed to update window shadow", error);
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

const getMonitorRect = async () => {
  const monitor = (await currentMonitor()) ?? (await primaryMonitor());
  if (!monitor) {
    return null;
  }
  const position = monitor.position;
  const size = monitor.size;
  if (!position || !size) {
    return null;
  }
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

const refreshEdgeInsets = async () => {
  const appWindow = getAppWindow();
  if (!appWindow) {
    return;
  }
  try {
    const [outerPosition, innerPosition, outerSize, innerSize] =
      await Promise.all([
        appWindow.outerPosition(),
        appWindow.innerPosition(),
        appWindow.outerSize(),
        appWindow.innerSize(),
      ]);
    const left = Math.max(0, innerPosition.x - outerPosition.x);
    const top = Math.max(0, innerPosition.y - outerPosition.y);
    const right = Math.max(0, outerSize.width - innerSize.width - left);
    const bottom = Math.max(0, outerSize.height - innerSize.height - top);
    edgeInsets.value = { left, right, top, bottom };
  } catch (error) {
    console.warn("Failed to resolve window insets", error);
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

const updateSnapAnchorToEdge = async () => {
  const [area, rect, monitorRect] = await Promise.all([
    getWorkAreaRect(),
    getWindowRect(),
    getMonitorRect(),
  ]);
  if (!area || !rect || !monitorRect) {
    return null;
  }
  const isLeft = rect.centerX <= area.left + area.width / 2;
  const kind = isLeft ? "edge-left" : "edge-right";
  const point = {
    x: isLeft ? monitorRect.left : monitorRect.right,
    y: rect.centerY,
  };
  edgeSide.value = isLeft ? "left" : "right";
  snapAnchor = { kind, point, edgeBounds: monitorRect };
  return snapAnchor;
};

const resolvePositionForAnchor = (anchor, area, size) => {
  const edgeBounds = anchor.edgeBounds ?? area;
  const { minX, minY, maxX, maxY } = getSnapBounds(area, size);
  let x = minX;
  let y = minY;
  const edgeOffset = Math.max(0, size.width - EDGE_LINE_THICKNESS);
  const leftInset = Math.max(edgeInsets.value.left, edgeInsetFallback.value);
  const rightInset = Math.max(edgeInsets.value.right, edgeInsetFallback.value);
  switch (anchor.kind) {
    case "edge-left":
      x = edgeBounds.left - edgeOffset - leftInset - EDGE_LEFT_NUDGE;
      y = anchor.point.y - size.height / 2;
      break;
    case "edge-right":
      x = edgeBounds.right - EDGE_LINE_THICKNESS - rightInset;
      y = anchor.point.y - size.height / 2;
      break;
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
  const edgeMinX = edgeBounds.left - edgeOffset - leftInset - EDGE_LEFT_NUDGE;
  const edgeMaxX = edgeBounds.right - EDGE_LINE_THICKNESS - rightInset;
  const clampedX =
    anchor.kind === "edge-left" || anchor.kind === "edge-right"
      ? clamp(x, edgeMinX, edgeMaxX)
      : clamp(x, minX, maxX);
  const clampedY = clamp(y, minY, maxY);
  return { x: Math.round(clampedX), y: Math.round(clampedY) };
};

const snapWindowToEdgesIfNeeded = async () => {
  if (isEdgeHidden.value) {
    return;
  }
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

const resolveCompactSize = () => {
  if (hideMode.value === "edge") {
    return {
      width: EDGE_LINE_SIZE.width,
      height: EDGE_LINE_SIZE.height,
    };
  }
  return COMPACT_SIZE;
};

const applyDesiredMode = async () => {
  if (resizeInFlight) {
    return;
  }
  resizeInFlight = true;
  while (true) {
    const nextCompact = desiredCompact;
    const nextSize = nextCompact ? resolveCompactSize() : fullSize.value;
    const nextMinSize =
      nextCompact && hideMode.value === "edge"
        ? new LogicalSize(EDGE_LINE_SIZE.width, EDGE_LINE_SIZE.height)
        : null;
    const nextMaxSize =
      nextCompact && hideMode.value === "edge"
        ? new LogicalSize(EDGE_LINE_SIZE.width, EDGE_LINE_SIZE.height)
        : null;
    const nextShadow = !(nextCompact && hideMode.value === "edge");
    const hasEdgeAnchor =
      snapAnchor &&
      (snapAnchor.kind === "edge-left" || snapAnchor.kind === "edge-right");
    if (hideMode.value === "edge") {
      if (!hasEdgeAnchor) {
        await updateSnapAnchorToEdge();
      }
    } else {
      await updateSnapAnchorFromWindow();
    }
    isCompact.value = nextCompact;
    await setWindowShadow(nextShadow);
    await setWindowMinSize(nextMinSize);
    await setWindowMaxSize(nextMaxSize);
    await setWindowSize(nextSize.width, nextSize.height);
    if (nextCompact && hideMode.value === "edge") {
      await refreshEdgeInsets();
    }
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

const isCursorNearEdgeLine = async () => {
  const [cursor, rect, monitorRect] = await Promise.all([
    cursorPosition(),
    getWindowRect(),
    getMonitorRect(),
  ]);
  if (!rect || !monitorRect) {
    return null;
  }
  const withinY =
    cursor.y >= rect.top - EDGE_ACTIVATE_PADDING &&
    cursor.y <= rect.bottom + EDGE_ACTIVATE_PADDING;
  if (!withinY) {
    return false;
  }
  if (edgeSide.value === "left") {
    return cursor.x <= monitorRect.left + EDGE_ACTIVATE_PADDING;
  }
  if (edgeSide.value === "right") {
    return cursor.x >= monitorRect.right - EDGE_ACTIVATE_PADDING;
  }
  return false;
};

const enterCompact = () => {
  requestCompactMode(true);
};

const exitCompact = () => {
  requestCompactMode(false);
};

const syncFullWidth = () => {
  const clamped = Math.round(clamp(fullWidth.value, FULL_WIDTH_MIN, FULL_WIDTH_MAX));
  if (clamped !== fullWidth.value) {
    fullWidth.value = clamped;
  }
  void applyDesiredMode();
};

const applyFullWidth = () => {
  fullWidth.value = fullWidthDraft.value;
  syncFullWidth();
};

const syncHideMode = () => {
  if (desiredCompact) {
    void applyDesiredMode();
  }
};

const applyHideMode = (mode) => {
  if (mode !== "compact") {
    return;
  }
  hideMode.value = mode;
  syncHideMode();
};

const minimizeToTray = async () => {
  try {
    await invoke("hide_main_window");
  } catch (error) {
    console.warn("Failed to hide window to tray", error);
  }
};

const handleMouseLeaveApp = () => {
  hideTooltip();
};

const openSettings = () => {
  settingsSection.value = "word-bank";
  isSettings.value = true;
  hideTooltip();
  exitCompact();
  void refreshWordBank();
};

const closeSettings = () => {
  isSettings.value = false;
  hideTooltip();
};

const setSettingsSection = (section) => {
  settingsSection.value = section;
  if (section !== "fuzzy-words") {
    fuzzyWordDetailId.value = null;
    stopAudioPlayback();
  }
  hideTooltip();
  if (section === "word-bank" || section === "dictionary-download") {
    void refreshWordBank();
  }
  if (section === "fuzzy-words") {
    fuzzyWordDetailId.value = null;
    void refreshFuzzyWords();
  }
  if (section === "study-calendar") {
    void fetchStudyCalendarCounts();
  }
  if (section === "dictionary-download") {
    dictionaryDownloadNotice.value = "";
  }
};

const setDictionaryState = (id, patch) => {
  const current = getDictionaryState(id);
  dictionaryDownloadStates.value = {
    ...dictionaryDownloadStates.value,
    [id]: { ...current, ...patch },
  };
};

const downloadDictionary = async (item) => {
  if (!item?.url || !item?.name) {
    return;
  }
  if (isDictionaryBusy(item.id)) {
    return;
  }
  dictionaryDownloadNotice.value = "";
  setDictionaryState(item.id, {
    status: "downloading",
    progress: 0,
    error: "",
    hasTotal: false,
  });
  try {
    const response = await fetch(item.url);
    if (!response.ok) {
      throw new Error("下载失败");
    }
    const total = Number(response.headers.get("content-length") ?? 0);
    let received = 0;
    let csvContent = "";
    if (response.body) {
      const reader = response.body.getReader();
      const decoder = new TextDecoder("utf-8");
      setDictionaryState(item.id, { hasTotal: total > 0 });
      while (true) {
        const { done, value } = await reader.read();
        if (done) {
          break;
        }
        if (value) {
          received += value.length;
          csvContent += decoder.decode(value, { stream: true });
          if (total > 0) {
            const progress = Math.min(99, Math.round((received / total) * 100));
            setDictionaryState(item.id, { progress });
          }
        }
      }
      csvContent += decoder.decode();
    } else {
      csvContent = await response.text();
    }
    setDictionaryState(item.id, { status: "importing", progress: 100 });
    await invoke("import_dictionary_csv", {
      name: item.name,
      csvContent,
      overwrite: isDictionaryDownloaded(item.name),
    });
    await refreshWordBank();
    setDictionaryState(item.id, { status: "done", progress: 100 });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    dictionaryDownloadNotice.value = message;
    setDictionaryState(item.id, { status: "failed", error: message });
  } finally {
    if (getDictionaryState(item.id).status === "downloading") {
      setDictionaryState(item.id, { status: "failed", error: "下载中断" });
    }
  }
};

const requestWordListCards = async () => {
  const lists = await invoke("list_word_lists");
  return Array.isArray(lists) ? lists : [];
};

const refreshWordBank = async () => {
  wordBankLoading.value = true;
  wordBankNotice.value = "";
  try {
    wordListCards.value = await requestWordListCards();
  } catch (error) {
    wordBankNotice.value = error instanceof Error ? error.message : String(error);
    wordListCards.value = [];
  } finally {
    wordBankLoading.value = false;
  }
};

const normalizeFuzzyWords = (value) => (Array.isArray(value) ? value : []);

const syncFuzzySelection = (nextWords) => {
  const ids = new Set(nextWords.map((item) => item.id));
  fuzzySelectedIds.value = fuzzySelectedIds.value.filter((id) => ids.has(id));
  if (!ids.has(fuzzyWordDetailId.value)) {
    fuzzyWordDetailId.value = null;
  }
};

const requestFuzzyWords = async () => {
  const words = await invoke("list_fuzzy_words", { sort: fuzzyWordSort.value });
  return normalizeFuzzyWords(words);
};

const refreshFuzzyWords = async () => {
  if (fuzzyWordsLoading.value) {
    return;
  }
  fuzzyWordsLoading.value = true;
  fuzzyWordsNotice.value = "";
  try {
    const words = await requestFuzzyWords();
    fuzzyWords.value = words;
    syncFuzzySelection(words);
  } catch (error) {
    fuzzyWordsNotice.value = error instanceof Error ? error.message : String(error);
    fuzzyWords.value = [];
    syncFuzzySelection([]);
  } finally {
    fuzzyWordsLoading.value = false;
  }
};

const toggleFuzzyWordSelection = (wordId) => {
  if (!wordId) {
    return;
  }
  if (fuzzySelectedIds.value.includes(wordId)) {
    fuzzySelectedIds.value = fuzzySelectedIds.value.filter((id) => id !== wordId);
  } else {
    fuzzySelectedIds.value = [...fuzzySelectedIds.value, wordId];
  }
};

const toggleFuzzySelectAll = () => {
  if (isFuzzyAllSelected.value) {
    fuzzySelectedIds.value = [];
  } else {
    fuzzySelectedIds.value = fuzzyWords.value.map((item) => item.id);
  }
};

const openFuzzyWordDetail = (word) => {
  if (!word?.id) {
    return;
  }
  stopAudioPlayback();
  fuzzyWordDetailId.value = word.id;
};

const closeFuzzyWordDetail = () => {
  fuzzyWordDetailId.value = null;
};

const stopAudioPlayback = () => {
  if (audioPlayer) {
    audioPlayer.pause();
    audioPlayer = null;
  }
  fuzzyAudioPlaying.value = null;
};

const formatPhonetic = (value) => {
  if (!value) {
    return " / - / ";
  }
  const trimmed = value.trim();
  const normalized = trimmed.replace(/^\[|\]$/g, "");
  return ` / ${normalized} / `;
};

const playAudio = (kind, url) => {
  if (!url) {
    return;
  }
  stopAudioPlayback();
  const player = new Audio(url);
  audioPlayer = player;
  fuzzyAudioPlaying.value = kind;
  player.addEventListener("ended", stopAudioPlayback, { once: true });
  player.addEventListener("pause", stopAudioPlayback, { once: true });
  player.addEventListener("error", stopAudioPlayback, { once: true });
  player.play().catch(() => {
    stopAudioPlayback();
  });
};

const clearFuzzyMarks = async (wordIds) => {
  if (!Array.isArray(wordIds) || wordIds.length === 0) {
    return;
  }
  fuzzyWordsNotice.value = "";
  try {
    await invoke("clear_fuzzy_marks", { wordIds });
    await refreshFuzzyWords();
    await refreshWordBank();
  } catch (error) {
    fuzzyWordsNotice.value = error instanceof Error ? error.message : String(error);
  }
};

const setFuzzySort = async (sort) => {
  if (!sort || fuzzyWordSort.value === sort) {
    return;
  }
  fuzzyWordSort.value = sort;
  await refreshFuzzyWords();
};

const invalidateStudyCalendarCache = () => {
  studyCalendarCache.loadedAt = 0;
};

const fetchStudyCalendarCounts = async (force = false) => {
  if (studyCalendarLoading.value) {
    return;
  }
  studyCalendarNotice.value = "";
  const now = Date.now();
  if (
    !force &&
    studyCalendarCache.loadedAt &&
    now - studyCalendarCache.loadedAt < STUDY_CALENDAR_CACHE_MS
  ) {
    studyCalendarCounts.value = [...studyCalendarCache.data];
    return;
  }
  studyCalendarLoading.value = true;
  try {
    const counts = await invoke("list_daily_study_counts");
    const normalized = Array.isArray(counts) ? counts : [];
    studyCalendarCounts.value = normalized;
    studyCalendarCache.data = normalized;
    studyCalendarCache.loadedAt = now;
  } catch (error) {
    studyCalendarNotice.value = error instanceof Error ? error.message : String(error);
    studyCalendarCounts.value = [];
  } finally {
    studyCalendarLoading.value = false;
  }
};

const setActiveWordList = async (listId) => {
  if (!listId) {
    return;
  }
  wordBankNotice.value = "";
  try {
    await invoke("set_active_word_list", { wordListId: listId });
    await refreshWordBank();
    void startLearningSession(true);
  } catch (error) {
    wordBankNotice.value = error instanceof Error ? error.message : String(error);
  }
};

const clearActiveWordList = async () => {
  wordBankNotice.value = "";
  try {
    await invoke("clear_active_word_list");
    await refreshWordBank();
    void startLearningSession(true);
  } catch (error) {
    wordBankNotice.value = error instanceof Error ? error.message : String(error);
  }
};

const deleteWordList = async (listId) => {
  if (!listId) {
    return;
  }
  wordBankNotice.value = "";
  try {
    await invoke("delete_word_list", { wordListId: listId });
    await refreshWordBank();
    void startLearningSession(true);
  } catch (error) {
    wordBankNotice.value = error instanceof Error ? error.message : String(error);
  }
};

const normalizeLearningWords = (value) => (Array.isArray(value) ? value : []);

const drawNextWord = () => {
  if (!remainingWords.value.length) {
    return null;
  }
  const index = Math.floor(Math.random() * remainingWords.value.length);
  const [next] = remainingWords.value.splice(index, 1);
  remainingWords.value = [...remainingWords.value];
  return next ?? null;
};

const applySessionWords = (words, resetHistory) => {
  remainingWords.value = [...words];
  if (resetHistory) {
    historyStack.value = [];
  }
};

const fetchLearningSession = async () => {
  const words = await invoke("allocate_learning_session");
  return normalizeLearningWords(words);
};

const prefetchNextSession = async () => {
  if (prefetchInFlight.value || prefetchWords.value.length > 0) {
    return;
  }
  if (remainingWords.value.length > 6) {
    return;
  }
  prefetchInFlight.value = true;
  try {
    const words = await fetchLearningSession();
    if (words.length) {
      prefetchWords.value = words;
    }
  } catch (_) {
    // ignore prefetch failures to keep the main flow responsive
  } finally {
    prefetchInFlight.value = false;
  }
};

const startLearningSession = async (resetHistory) => {
  if (learningBusy.value) {
    return false;
  }
  if (!hasActiveWordList.value) {
    learningNotice.value = "";
    currentWord.value = null;
    remainingWords.value = [];
    prefetchWords.value = [];
    if (resetHistory) {
      historyStack.value = [];
    }
    return false;
  }
  learningBusy.value = true;
  learningNotice.value = "";
  try {
    const words = await fetchLearningSession();
    if (!words.length) {
      learningNotice.value = "当前词库暂无单词";
      currentWord.value = null;
      remainingWords.value = [];
      return false;
    }
    applySessionWords(words, resetHistory);
    currentWord.value = drawNextWord();
    prefetchWords.value = [];
    void prefetchNextSession();
    return true;
  } catch (error) {
    learningNotice.value = error instanceof Error ? error.message : String(error);
    currentWord.value = null;
    remainingWords.value = [];
    return false;
  } finally {
    learningBusy.value = false;
  }
};

const ensureNextWord = async () => {
  let nextWord = drawNextWord();
  if (!nextWord) {
    if (prefetchWords.value.length) {
      applySessionWords(prefetchWords.value, false);
      prefetchWords.value = [];
      nextWord = drawNextWord();
    } else {
      const words = await fetchLearningSession();
      if (words.length) {
        applySessionWords(words, false);
        nextWord = drawNextWord();
      }
    }
  }
  currentWord.value = nextWord;
  if (!nextWord && !learningNotice.value) {
    learningNotice.value = "当前词库暂无单词";
  }
  if (nextWord) {
    void prefetchNextSession();
  }
};

const goNext = async () => {
  if (learningBusy.value || !hasActiveWordList.value) {
    return;
  }
  learningNotice.value = "";
  if (!currentWord.value) {
    await startLearningSession(true);
    return;
  }
  learningBusy.value = true;
  try {
    const progress = await invoke("increment_proficiency", {
      wordId: currentWord.value.id,
    });
    historyStack.value = [
      ...historyStack.value,
      {
        ...currentWord.value,
        proficiency_score: progress.proficiency_score,
      },
    ];
    invalidateStudyCalendarCache();
    await ensureNextWord();
  } catch (error) {
    learningNotice.value = error instanceof Error ? error.message : String(error);
  } finally {
    learningBusy.value = false;
  }
};

const goPrevious = () => {
  if (!canGoPrevious.value || !hasActiveWordList.value) {
    return;
  }
  learningNotice.value = "";
  const next = historyStack.value[historyStack.value.length - 1];
  historyStack.value = historyStack.value.slice(0, -1);
  currentWord.value = next ?? null;
};

const markFuzzy = async () => {
  if (!currentWord.value || learningBusy.value || !hasActiveWordList.value) {
    return;
  }
  learningNotice.value = "";
  learningBusy.value = true;
  try {
    const progress = await invoke("decrement_proficiency", {
      wordId: currentWord.value.id,
    });
    currentWord.value = {
      ...currentWord.value,
      proficiency_score: progress.proficiency_score,
    };
    invalidateStudyCalendarCache();
  } catch (error) {
    learningNotice.value = error instanceof Error ? error.message : String(error);
  } finally {
    learningBusy.value = false;
  }
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
  const position = target.dataset.tooltipPosition ?? "right";
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
    if (isSettings.value) {
      if (desiredCompact) {
        exitCompact();
      }
      return;
    }
    if (isEdgeHidden.value) {
      const isNearEdge = await isCursorNearEdgeLine();
      if (isNearEdge == null) {
        return;
      }
      if (isNearEdge) {
        if (desiredCompact) {
          edgeRevealUntil = Date.now() + EDGE_REVEAL_HOLD_MS;
          exitCompact();
        }
        return;
      }
      if (!desiredCompact) {
        enterCompact();
      }
      return;
    }
    if (hideMode.value === "edge" && Date.now() < edgeRevealUntil) {
      return;
    }
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
  const fallback = await getPhysicalLengthFromLogical(EDGE_FRAME_FALLBACK);
  edgeInsetFallback.value = fallback ?? EDGE_FRAME_FALLBACK;
  desiredCompact = true;
  await applyDesiredMode();
  await refreshWordBank();
  setCalendarAnchor(new Date());
  const appWindow = getAppWindow();
  if (appWindow) {
    unlistenMove = await appWindow.onMoved(() => {
      if (!isRepositioning) {
        if (hideMode.value === "edge") {
          void updateSnapAnchorToEdge();
        } else {
          scheduleSnapToEdges();
        }
      }
    });
    unlistenFocus = await appWindow.onFocusChanged(({ payload }) => {
      if (!payload) {
        if (!isSettings.value) {
          enterCompact();
        }
      }
    });
  }
  unlistenHideMode = await listen("hide-mode-change", (event) => {
    applyHideMode(event.payload);
  });
  cursorPollTimer = window.setInterval(
    updateCompactFromCursor,
    CURSOR_POLL_INTERVAL_MS
  );
  void startLearningSession(true);
});

onBeforeUnmount(() => {
  if (unlistenMove) {
    unlistenMove();
  }
  if (unlistenFocus) {
    unlistenFocus();
  }
  if (unlistenHideMode) {
    unlistenHideMode();
  }
  if (cursorPollTimer) {
    window.clearInterval(cursorPollTimer);
  }
  if (snapDebounceTimer) {
    window.clearTimeout(snapDebounceTimer);
  }
  stopAudioPlayback();
});
</script>

<template>
  <div
    class="app"
    :class="{
      'is-compact': isCompact,
      'is-edge-hidden': isEdgeHidden,
      'edge-left': edgeSide === 'left',
      'edge-right': edgeSide === 'right',
    }"
    @mouseleave="handleMouseLeaveApp"
  >
    <div v-if="isCompact" class="view view-compact">
      <div
        v-if="isEdgeHidden"
        class="edge-shell"
        @mousedown="handleDragStart"
        @mouseenter="exitCompact"
      >
        <span class="edge-line" aria-hidden="true"></span>
      </div>
      <div v-else class="compact-shell" @mousedown="handleDragStart">
        <span class="word word-compact">{{ displayWord }}</span>
      </div>
    </div>

    <div v-else class="view view-main">
      <div class="view-main-scale" :style="uiScaleStyle">
        <main v-if="!isSettings" class="card">
        <div class="top-row" @mousedown="handleDragStart">
          <div
            class="proficiency-box"
            :class="`level-${proficiencyLevel}`"
            :aria-label="proficiencyLabel"
          ></div>
          <div class="top-actions">
            <button
              class="settings-button icon-button minimize-button"
              type="button"
              aria-label="最小化到托盘"
              @click="minimizeToTray"
              @mousedown.stop
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <path d="M6 12h12" />
              </svg>
            </button>
            <button
              class="settings-button icon-button"
              type="button"
              aria-label="设置"
              @click="openSettings"
              @mousedown.stop
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
        </div>

        <template v-if="!showEmptyState">
          <div class="word-line">
            <span class="word">{{ displayWord }}</span>
            <span v-if="displayPhonetic" class="phonetic">{{ displayPhonetic }}</span>
            <div v-if="currentWord" class="word-audio">
              <button
                class="word-audio-button"
                type="button"
                :class="{ 'is-playing': fuzzyAudioPlaying === 'main-uk' }"
                :disabled="!currentWord.audio_uk"
                @click="playAudio('main-uk', currentWord.audio_uk)"
              >
                英
                <span class="word-audio-icon">&#x266A;</span>
              </button>
              <button
                class="word-audio-button"
                type="button"
                :class="{ 'is-playing': fuzzyAudioPlaying === 'main-us' }"
                :disabled="!currentWord.audio_us"
                @click="playAudio('main-us', currentWord.audio_us)"
              >
                美
                <span class="word-audio-icon">&#x266A;</span>
              </button>
            </div>
          </div>

          <div class="detail-group">
            <p v-if="displayMeaning" class="word-cn">{{ displayMeaning }}</p>

            <div v-if="displayExample || displayExampleTranslation" class="example-group">
              <p v-if="displayExample" class="example">{{ displayExample }}</p>
              <p v-if="displayExampleTranslation" class="example-cn">
                {{ displayExampleTranslation }}
              </p>
            </div>

            <p v-if="learningNotice" class="learning-notice">{{ learningNotice }}</p>
          </div>

          <div class="nav-actions">
            <button
              class="nav-button"
              type="button"
              :disabled="!canGoPrevious"
              @click="goPrevious"
            >
              上一个
            </button>
            <button
              class="nav-button"
              type="button"
              :disabled="!canMarkFuzzy"
              @click="markFuzzy"
            >
              模糊
            </button>
            <button
              class="nav-button"
              type="button"
              :disabled="!canGoNext"
              @click="goNext"
            >
              {{ nextLabel }}
            </button>
          </div>
        </template>
        <div v-else class="empty-state">
          <p class="empty-title">未选择词库</p>
          <p class="empty-desc">请前往设置选择或创建词库。</p>
        </div>
        </main>
        <section v-else class="settings">
        <header class="settings-header" @mousedown="handleDragStart">
          <button
            class="back-button icon-button"
            type="button"
            aria-label="返回"
            @click="closeSettings"
            @mousedown.stop
            @mouseenter="showTooltip"
            @mouseleave="hideTooltip"
            data-tooltip="返回"
          >
            <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
              <path d="M13 6l-6 6 6 6" />
              <path d="M7 12h10" />
            </svg>
          </button>
          <span class="settings-title">设置</span>
        </header>
        <div class="settings-body">
          <nav class="settings-nav" aria-label="设置导航">
            <button
              class="settings-nav-item icon-button"
              :class="{ 'is-active': settingsSection === 'word-bank' }"
              type="button"
              @click="setSettingsSection('word-bank')"
              @mousedown.stop
              @mouseenter="showTooltip"
              @mouseleave="hideTooltip"
              aria-label="词库"
              data-tooltip="词库"
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
              aria-label="&#x6A21;&#x7CCA;&#x8BCD;"
              data-tooltip="&#x6A21;&#x7CCA;&#x8BCD;"
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
              aria-label="学习日历"
              data-tooltip="学习日历"
              data-tooltip-position="right"
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <rect x="4" y="6" width="16" height="14" rx="2" />
                <path d="M8 3v4M16 3v4M4 10h16" />
              </svg>
            </button>
            <button
              class="settings-nav-item icon-button"
              :class="{ 'is-active': settingsSection === 'dictionary-download' }"
              type="button"
              @click="setSettingsSection('dictionary-download')"
              @mousedown.stop
              @mouseenter="showTooltip"
              @mouseleave="hideTooltip"
              aria-label="词典下载"
              data-tooltip="词典下载"
              data-tooltip-position="right"
            >
              <svg aria-hidden="true" viewBox="0 0 24 24" focusable="false">
                <path d="M12 4v10" />
                <path d="M7 11l5 5 5-5" />
                <path d="M5 20h14" />
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
              aria-label="更多"
              data-tooltip="更多"
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
            <div
              v-if="settingsSection === 'word-bank'"
              class="word-bank-panel"
            >
              <div class="word-bank-header">
                <span class="word-bank-title">词库导航</span>
                <span v-if="wordBankLoading" class="word-bank-status">加载中...</span>
              </div>
              <p v-if="wordBankNotice" class="word-bank-notice">{{ wordBankNotice }}</p>
              <p
                v-else-if="!sortedWordListCards.length && !wordBankLoading"
                class="word-bank-empty"
              >
                暂无词库
              </p>
              <div v-if="sortedWordListCards.length" class="word-list-grid">
                <div
                  v-for="list in sortedWordListCards"
                  :key="list.id"
                  class="word-list-card"
                  :class="{ 'is-active': list.is_active }"
                >
                  <div class="word-list-meta">
                    <div class="word-list-title-row">
                      <div class="word-list-title">{{ list.name }}</div>
                      <span v-if="list.is_system" class="word-list-tag">系统</span>
                    </div>
                    <div class="word-list-count">{{ list.word_count }} 个单词</div>
                  </div>
                  <div class="word-list-actions">
                    <button
                      class="word-list-action"
                      type="button"
                      @click="
                        list.is_active
                          ? clearActiveWordList()
                          : setActiveWordList(list.id)
                      "
                    >
                      {{ list.is_active ? "取消使用" : "使用" }}
                    </button>
                    <button
                      v-if="!list.is_active && !list.is_system"
                      class="word-list-action word-list-delete"
                      type="button"
                      @click="deleteWordList(list.id)"
                    >
                      删除
                    </button>
                  </div>
                </div>
              </div>
            </div>
            <div
              v-else-if="settingsSection === 'fuzzy-words'"
              class="fuzzy-words"
            >
              <div v-if="fuzzyWordDetail" class="fuzzy-detail-page">
                <div class="fuzzy-detail-header">
                </div>
                <div class="fuzzy-detail-body">
                  <span class="fuzzy-detail-word">{{ fuzzyWordDetail.word }}</span>
                  <div class="fuzzy-detail-audio">
                    <button
                      class="fuzzy-audio-pill"
                      type="button"
                      :class="{ 'is-playing': fuzzyAudioPlaying === 'uk' }"
                      :disabled="!fuzzyWordDetail.audio_uk"
                      @click="playAudio('uk', fuzzyWordDetail.audio_uk)"
                    >
                      <span class="fuzzy-audio-label">英</span>
                      <span class="fuzzy-audio-phonetic">
                        {{ formatPhonetic(fuzzyWordDetail.phonetic) }}
                      </span>
                      <span class="fuzzy-audio-icon">&#x266A;</span>
                    </button>
                    <button
                      class="fuzzy-audio-pill"
                      type="button"
                      :class="{ 'is-playing': fuzzyAudioPlaying === 'us' }"
                      :disabled="!fuzzyWordDetail.audio_us"
                      @click="playAudio('us', fuzzyWordDetail.audio_us)"
                    >
                      <span class="fuzzy-audio-label">美</span>
                      <span class="fuzzy-audio-phonetic">
                        {{ formatPhonetic(fuzzyWordDetail.phonetic) }}
                      </span>
                      <span class="fuzzy-audio-icon">&#x266A;</span>
                    </button>
                  </div>
                  <p
                    v-if="fuzzyWordDetail.part_of_speech_and_meanings"
                    class="fuzzy-detail-meaning"
                  >
                    {{ fuzzyWordDetail.part_of_speech_and_meanings }}
                  </p>
                  <div
                    v-if="
                      fuzzyWordDetail.example_sentence ||
                      fuzzyWordDetail.example_translation
                    "
                    class="fuzzy-detail-examples"
                  >
                    <p v-if="fuzzyWordDetail.example_sentence" class="example">
                      {{ fuzzyWordDetail.example_sentence }}
                    </p>
                    <p
                      v-if="fuzzyWordDetail.example_translation"
                      class="example-cn"
                    >
                      {{ fuzzyWordDetail.example_translation }}
                    </p>
                  </div>
                </div>
              </div>
              <template v-else>
                <div class="fuzzy-words-header">
                  <div class="fuzzy-words-title-row">
                    <span class="fuzzy-words-title">模糊词</span>
                    <span v-if="fuzzyWordsLoading" class="fuzzy-words-status">
                      加载中...
                    </span>
                  </div>
                  <div class="fuzzy-sort-toggle" role="group" aria-label="排序">
                    <button
                      type="button"
                      class="fuzzy-sort-button"
                      :class="{ 'is-active': fuzzyWordSort === 'marked' }"
                      @click="setFuzzySort('marked')"
                    >
                      标记时间
                    </button>
                    <button
                      type="button"
                      class="fuzzy-sort-button"
                      :class="{ 'is-active': fuzzyWordSort === 'alpha' }"
                      @click="setFuzzySort('alpha')"
                    >
                      字母
                    </button>
                  </div>
                </div>
                <div class="fuzzy-words-toolbar">
                  <label class="fuzzy-select-all">
                    <input
                      type="checkbox"
                      :checked="isFuzzyAllSelected"
                      @change="toggleFuzzySelectAll"
                    />
                    <span>全选</span>
                  </label>
                  <button
                    class="fuzzy-clear-button"
                    type="button"
                    :disabled="!hasFuzzySelection"
                    @click="clearFuzzyMarks(fuzzySelectedIds)"
                  >
                    取消模糊
                  </button>
                  <span class="fuzzy-words-count">{{ fuzzyWords.length }} 个</span>
                </div>
                <p v-if="fuzzyWordsNotice" class="fuzzy-words-notice">
                  {{ fuzzyWordsNotice }}
                </p>
                <p
                  v-else-if="!fuzzyWords.length && !fuzzyWordsLoading"
                  class="fuzzy-words-empty"
                >
                  暂无模糊词
                </p>
                <div v-else class="fuzzy-words-body">
                  <div class="fuzzy-words-list">
                    <div
                      v-for="item in fuzzyWords"
                      :key="item.id"
                      class="fuzzy-word-row"
                      @click="openFuzzyWordDetail(item)"
                    >
                      <label class="fuzzy-word-checkbox" @click.stop>
                        <input
                          type="checkbox"
                          :checked="fuzzySelectedIds.includes(item.id)"
                          @change="toggleFuzzyWordSelection(item.id)"
                        />
                      </label>
                      <div class="fuzzy-word-meta">
                        <div class="fuzzy-word-text">{{ item.word }}</div>
                        <div
                          v-if="item.part_of_speech_and_meanings"
                          class="fuzzy-word-meaning"
                        >
                          {{ item.part_of_speech_and_meanings }}
                        </div>
                      </div>
                      <button
                        class="fuzzy-word-action"
                        type="button"
                        @click.stop="clearFuzzyMarks([item.id])"
                      >
                        取消
                      </button>
                    </div>
                  </div>
                </div>
              </template>
            </div>
            <div
              v-else-if="settingsSection === 'study-calendar'"
              class="study-calendar"
            >
              <div class="study-calendar-header">
                <span class="study-calendar-title">学习日历</span>
                <div class="study-calendar-toggle" role="group" aria-label="视图切换">
                  <button
                    type="button"
                    class="study-calendar-toggle-button"
                    :class="{ 'is-active': studyCalendarView === 'calendar' }"
                    @click="studyCalendarView = 'calendar'"
                  >
                    日历
                  </button>
                  <button
                    type="button"
                    class="study-calendar-toggle-button"
                    :class="{ 'is-active': studyCalendarView === 'line' }"
                    @click="studyCalendarView = 'line'"
                  >
                    折线
                  </button>
                </div>
              </div>
              <p v-if="studyCalendarLoading" class="study-calendar-status">加载中...</p>
              <p v-else-if="studyCalendarNotice" class="study-calendar-notice">
                {{ studyCalendarNotice }}
              </p>
              <div v-else class="study-calendar-body">
                <div class="study-calendar-month-nav">
                  <button
                    type="button"
                    class="study-calendar-nav-button"
                    :disabled="!canGoPrevMonth"
                    @click="shiftCalendarMonth(-1)"
                  >
                    上一月
                  </button>
                  <span class="study-calendar-month-label">
                    {{ calendarMonthLabel }}
                  </span>
                  <button
                    type="button"
                    class="study-calendar-nav-button"
                    :disabled="!canGoNextMonth"
                    @click="shiftCalendarMonth(1)"
                  >
                    下一月
                  </button>
                </div>
                <template v-if="studyCalendarView === 'calendar'">
                  <div class="study-calendar-weekdays">
                    <span
                      v-for="weekday in STUDY_CALENDAR_WEEKDAYS"
                      :key="weekday"
                      class="study-calendar-weekday"
                    >
                      {{ weekday }}
                    </span>
                  </div>
                  <div class="study-calendar-grid">
                    <div
                      v-for="cell in calendarCells"
                      :key="cell.key"
                      class="study-calendar-cell"
                      :class="{
                        'has-study': cell.count > 0,
                        'is-outside': !cell.isCurrentMonth,
                        'is-today': cell.isToday,
                      }"
                    >
                      <span class="study-calendar-date">{{ cell.label }}</span>
                      <span v-if="cell.count > 0" class="study-calendar-count">
                        {{ cell.count }}
                      </span>
                    </div>
                  </div>
                </template>
                <template v-else>
                  <div class="study-chart">
                    <svg
                      :viewBox="`0 0 ${STUDY_CHART_SIZE.width} ${STUDY_CHART_SIZE.height}`"
                      preserveAspectRatio="none"
                      role="img"
                      aria-label="每日学习单词数量折线图"
                      @mousemove="handleStudyChartMove"
                      @mouseleave="hideStudyChartTooltip"
                    >
                      <line
                        class="study-chart-axis"
                        :x1="STUDY_CHART_PADDING.left"
                        :y1="STUDY_CHART_PADDING.top"
                        :x2="STUDY_CHART_PADDING.left"
                        :y2="STUDY_CHART_SIZE.height - STUDY_CHART_PADDING.bottom"
                      />
                      <line
                        class="study-chart-axis"
                        :x1="STUDY_CHART_PADDING.left"
                        :y1="STUDY_CHART_SIZE.height - STUDY_CHART_PADDING.bottom"
                        :x2="STUDY_CHART_SIZE.width - STUDY_CHART_PADDING.right"
                        :y2="STUDY_CHART_SIZE.height - STUDY_CHART_PADDING.bottom"
                      />
                      <g v-for="tick in studyChartYTicks" :key="tick.value">
                        <line
                          class="study-chart-grid"
                          :x1="STUDY_CHART_PADDING.left"
                          :y1="tick.y"
                          :x2="STUDY_CHART_SIZE.width - STUDY_CHART_PADDING.right"
                          :y2="tick.y"
                        />
                        <text
                          class="study-chart-label"
                          :x="STUDY_CHART_PADDING.left - 6"
                          :y="tick.y + 3"
                          text-anchor="end"
                        >
                          {{ tick.label }}
                        </text>
                      </g>
                      <polyline
                        v-if="studyChartPoints"
                        class="study-chart-line"
                        :points="studyChartPoints"
                      />
                      <g v-for="label in studyChartXAxisLabels" :key="label.x">
                        <text
                          class="study-chart-label"
                          :x="label.x"
                          :y="STUDY_CHART_SIZE.height - 6"
                          text-anchor="middle"
                        >
                          {{ label.label }}
                        </text>
                      </g>
                    </svg>
                    <div
                      v-if="studyChartHover.visible"
                      class="study-chart-tooltip"
                      :style="studyChartTooltipStyle"
                    >
                      {{ studyChartHover.value }}
                    </div>
                  </div>
                </template>
              </div>
            </div>
                        <div
              v-else-if="settingsSection === 'dictionary-download'"
              class="dictionary-download"
            >
              <div class="dictionary-download-header">
                <span class="dictionary-download-title">词典下载</span>
              </div>
              <p class="dictionary-download-desc">
                当前提供以下词典链接，点击即可下载。
              </p>
              <p
                v-if="dictionaryDownloadNotice"
                class="dictionary-download-notice"
              >
                {{ dictionaryDownloadNotice }}
              </p>
              <div class="dictionary-download-list">
                <div
                  v-for="item in dictionaryDownloads"
                  :key="item.id"
                  class="dictionary-download-item"
                >
                  <div class="dictionary-download-meta">
                    <span class="dictionary-download-name">{{ item.name }}</span>
                    <span
                      class="dictionary-download-state"
                      :class="{ 'is-downloaded': isDictionaryDownloaded(item.name) }"
                    >
                      {{ isDictionaryDownloaded(item.name) ? "已下载" : "未下载" }}
                    </span>
                    <span
                      v-if="getDictionaryState(item.id).status === 'failed'"
                      class="dictionary-download-error"
                    >
                      {{ getDictionaryState(item.id).error || "下载失败" }}
                    </span>
                  </div>
                  <button
                    class="dictionary-download-action"
                    type="button"
                    :disabled="isDictionaryBusy(item.id)"
                    @click="downloadDictionary(item)"
                  >
                    {{
                      getDictionaryState(item.id).status === "failed"
                        ? "重试下载"
                        : isDictionaryDownloaded(item.name)
                          ? getDictionaryState(item.id).status === "importing"
                            ? "导入中..."
                            : getDictionaryState(item.id).status === "downloading"
                              ? getDictionaryState(item.id).hasTotal
                                ? `下载中 ${getDictionaryState(item.id).progress}%`
                                : "下载中..."
                              : "重新下载覆盖"
                          : getDictionaryState(item.id).status === "downloading"
                            ? getDictionaryState(item.id).hasTotal
                              ? `下载中 ${getDictionaryState(item.id).progress}%`
                              : "下载中..."
                            : getDictionaryState(item.id).status === "importing"
                              ? "导入中..."
                              : "下载"
                    }}
                  </button>
                </div>
              </div>
            </div>
            <div v-else-if="settingsSection === 'more'" class="settings-more">
              <button
                class="settings-more-item"
                type="button"
                @click="setSettingsSection('more-window-size')"
              >
                <span>窗口大小调整</span>
                <span class="settings-more-arrow">></span>
              </button>
            </div>
            <div
              v-else-if="settingsSection === 'more-window-size'"
              class="settings-more-detail"
            >
              <div class="settings-more-header">
                <button
                  class="settings-more-back"
                  type="button"
                  @click="setSettingsSection('more')"
                >
                  返回
                </button>
                <span class="settings-more-title">窗口大小调整</span>
              </div>
              <div class="settings-more-card">
                <p class="settings-more-label">全尺寸窗口宽度</p>
                <div class="settings-more-slider">
                  <input
                    v-model.number="fullWidthDraft"
                    type="range"
                    :min="FULL_WIDTH_MIN"
                    :max="FULL_WIDTH_MAX"
                    step="1"
                    @change="applyFullWidth"
                  />
                  <span class="settings-more-value">{{ fullSizeDraftLabel }}</span>
                </div>
                <p class="settings-more-hint">
                  最大宽度 450px，按当前比例自动调整高度。
                </p>
              </div>
            </div>
            <p v-else class="settings-placeholder">该模块正在完善中。</p>
          </div>
        </div>
          <div
            v-if="tooltip.visible"
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
  --icon-size: 24px;
  --nav-icon-size: 34px;
  --nav-column: 56px;
  --tooltip-font: 0.42rem;
}

.app.is-compact {
  padding: 0;
}

.app.is-edge-hidden {
  border: none;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
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
  align-items: start;
  justify-items: start;
}

.view-main-scale {
  width: var(--ui-base-width, 334px);
  height: var(--ui-base-height, 139px);
  transform: scale(var(--ui-scale, 1));
  transform-origin: top left;
}

.view-compact {
  height: 100%;
}

.edge-shell {
  height: 100%;
  width: 100%;
  display: flex;
  align-items: stretch;
  justify-content: center;
  cursor: pointer;
}

.app.edge-left .edge-shell {
  justify-content: flex-start;
}

.app.edge-right .edge-shell {
  justify-content: flex-end;
}

.edge-line {
  width: 100%;
  height: 100%;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.55);
}

.app.edge-right .edge-line {
  box-shadow: -1px 0 0 rgba(255, 255, 255, 0.9);
}

.app.edge-left .edge-line {
  box-shadow: 5px 0 0 rgba(255, 255, 255, 0.95);
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
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  gap: 4px;
  padding: 0;
  margin-top: -5px;
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

.top-actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
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
  position: relative;
  top: 5px;
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

.minimize-button {
  --icon-glyph-size: calc(var(--icon-size) * 0.5);
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

.word-audio {
  display: inline-flex;
  gap: 4px;
  align-items: center;
}

.word-audio-button {
  border-radius: 999px;
  border: 1px solid rgba(31, 29, 26, 0.08);
  background: #f4f5f7;
  font-size: 0.45rem;
  font-weight: 600;
  padding: 2px 6px;
  color: #1f1d1a;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.word-audio-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.word-audio-button.is-playing {
  border-color: rgba(255, 75, 75, 0.35);
  box-shadow: 0 4px 10px -8px rgba(255, 75, 75, 0.6);
}

.word-audio-icon {
  font-size: 0.48rem;
  color: #ff4b4b;
}

.word-audio-button.is-playing .word-audio-icon {
  animation: fuzzy-audio-pulse 0.9s ease-in-out infinite;
}

.word-cn {
  margin: 0;
  font-size: 0.65rem;
  line-height: 1.3;
  color: #2a2723;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.learning-notice {
  margin: 0;
  font-size: 0.52rem;
  color: #9b1c1c;
}

.detail-group {
  display: grid;
  gap: 2px;
  min-height: 0;
  overflow: hidden;
  align-content: start;
}

.empty-state {
  display: grid;
  gap: 4px;
  align-content: center;
  justify-items: center;
  text-align: center;
  min-height: 60px;
}

.empty-title {
  margin: 0;
  font-size: 0.7rem;
  color: #1f1d1a;
  font-weight: 600;
}

.empty-desc {
  margin: 0;
  font-size: 0.55rem;
  color: var(--muted);
}

.example-group {
  display: grid;
  gap: 2px;
  align-content: start;
  margin-top: 2px;
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
  grid-template-columns: repeat(3, minmax(0, 1fr));
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

.study-calendar {
  display: grid;
  gap: 8px;
  align-content: start;
}

.study-calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.study-calendar-title {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.study-calendar-toggle {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
}

.study-calendar-toggle-button {
  border: none;
  background: transparent;
  font-size: 0.5rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  font-weight: 600;
  color: #1f1d1a;
  padding: 4px 6px;
  border-radius: 8px;
  cursor: pointer;
}

.study-calendar-toggle-button.is-active {
  background: #1b9aaa;
  color: #fff;
}

.study-calendar-status,
.study-calendar-notice {
  margin: 0;
  font-size: 0.52rem;
  color: var(--muted);
}

.study-calendar-notice {
  color: #9b1c1c;
}

.study-calendar-body {
  display: grid;
  gap: 6px;
}

.study-calendar-month-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.study-calendar-nav-button {
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.8);
  font-size: 0.48rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 4px 6px;
  cursor: pointer;
  color: #1f1d1a;
}

.study-calendar-nav-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.study-calendar-month-label {
  font-size: 0.54rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.study-calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  text-align: center;
  font-size: 0.45rem;
  color: var(--muted);
}

.study-calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 4px;
}

.study-calendar-cell {
  position: relative;
  min-height: 26px;
  border-radius: 6px;
  padding: 2px 3px;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(31, 29, 26, 0.08);
  font-size: 0.48rem;
  text-align: right;
}

.study-calendar-cell.is-outside {
  opacity: 0.35;
}

.study-calendar-cell.has-study {
  background: #dff5e1;
  border-color: rgba(93, 168, 116, 0.35);
}

.study-calendar-date {
  display: inline-block;
}

.study-calendar-count {
  position: absolute;
  left: 3px;
  bottom: 2px;
  font-size: 0.42rem;
  font-weight: 600;
  color: #2f6b3d;
}

.study-chart {
  position: relative;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
  padding: 6px;
}

.study-chart svg {
  width: 100%;
  height: 140px;
  display: block;
}

.study-chart-axis {
  stroke: rgba(31, 29, 26, 0.25);
  stroke-width: 1;
}

.study-chart-grid {
  stroke: rgba(31, 29, 26, 0.08);
  stroke-width: 1;
}

.study-chart-line {
  fill: none;
  stroke: #1b9aaa;
  stroke-width: 2;
}

.study-chart-label {
  fill: var(--muted);
  font-size: 0.45rem;
}

.study-chart-tooltip {
  position: absolute;
  padding: 2px 6px;
  border-radius: 6px;
  background: #1f1d1a;
  color: #fff;
  font-size: 0.45rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  transform: translate(-50%, -100%);
  pointer-events: none;
  white-space: nowrap;
}

.settings-more {
  display: grid;
  gap: 6px;
  align-content: start;
}

.settings-more-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.75);
  font-size: 0.55rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  color: #1f1d1a;
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  box-shadow: 0 8px 12px -14px var(--shadow);
}

.settings-more-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 14px -14px var(--shadow);
}

.settings-more-arrow {
  font-size: 0.7rem;
  color: var(--muted);
}

.settings-more-detail {
  display: grid;
  gap: 8px;
  align-content: start;
}

.settings-more-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-more-back {
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 0.5rem;
  letter-spacing: 0.06em;
  cursor: pointer;
}

.settings-more-title {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-more-card {
  border-radius: 12px;
  border: 1px solid var(--stroke);
  padding: 10px;
  background: rgba(255, 255, 255, 0.7);
  display: grid;
  gap: 8px;
}

.settings-more-label {
  margin: 0;
  font-size: 0.54rem;
  font-weight: 600;
  letter-spacing: 0.06em;
}

.settings-more-slider {
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-more-slider input[type="range"] {
  flex: 1;
}

.settings-more-value {
  min-width: 86px;
  text-align: right;
  font-size: 0.5rem;
  color: var(--muted);
}

.settings-more-option {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  font-size: 0.54rem;
  color: #1f1d1a;
}

.settings-more-hint {
  margin: 0;
  font-size: 0.5rem;
  color: var(--muted);
}

.word-bank-panel {
  display: grid;
  gap: 8px;
  align-content: start;
}

.word-bank-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.word-bank-title {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.word-bank-status {
  font-size: 0.5rem;
  color: var(--muted);
}

.word-bank-notice {
  margin: 0;
  font-size: 0.52rem;
  color: #9b1c1c;
}

.word-bank-empty {
  margin: 0;
  font-size: 0.52rem;
  color: var(--muted);
}

.word-list-grid {
  display: grid;
  gap: 6px;
}

.word-list-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.8);
}

.word-list-card.is-active {
  border-color: rgba(27, 154, 170, 0.4);
  box-shadow: 0 10px 16px -16px rgba(27, 154, 170, 0.5);
}

.word-list-meta {
  display: grid;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.word-list-title-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.word-list-title {
  font-size: 0.58rem;
  font-weight: 600;
}

.word-list-tag {
  font-size: 0.42rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 1px 5px;
  border-radius: 999px;
  border: 1px solid rgba(27, 154, 170, 0.35);
  background: rgba(27, 154, 170, 0.12);
  color: #145e67;
}

.word-list-count {
  font-size: 0.5rem;
  color: var(--muted);
}

.word-list-actions {
  display: grid;
  gap: 6px;
  justify-items: end;
}

.word-list-action {
  padding: 4px 10px;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.85);
  font-size: 0.5rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  color: #1f1d1a;
  cursor: pointer;
}

.word-list-delete {
  color: #b42318;
  border-color: rgba(180, 35, 24, 0.35);
  background: rgba(255, 237, 235, 0.8);
}

.word-list-action:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.fuzzy-words {
  display: grid;
  gap: 8px;
  align-content: start;
}

.fuzzy-words-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.fuzzy-words-title-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.fuzzy-words-title {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.fuzzy-words-status {
  font-size: 0.5rem;
  color: var(--muted);
}

.fuzzy-sort-toggle {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
}

.fuzzy-sort-button {
  border: none;
  background: transparent;
  font-size: 0.5rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  font-weight: 600;
  color: #1f1d1a;
  padding: 4px 6px;
  border-radius: 8px;
  cursor: pointer;
}

.fuzzy-sort-button.is-active {
  background: rgba(27, 154, 170, 0.16);
  color: #145e67;
}

.fuzzy-words-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.fuzzy-select-all {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 0.5rem;
  color: #1f1d1a;
}

.fuzzy-clear-button {
  padding: 4px 10px;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.85);
  font-size: 0.5rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  color: #1f1d1a;
  cursor: pointer;
}

.fuzzy-clear-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.fuzzy-words-count {
  margin-left: auto;
  font-size: 0.5rem;
  color: var(--muted);
}

.fuzzy-words-notice {
  margin: 0;
  font-size: 0.52rem;
  color: #9b1c1c;
}

.fuzzy-words-empty {
  margin: 0;
  font-size: 0.52rem;
  color: var(--muted);
}

.fuzzy-words-body {
  display: grid;
  gap: 8px;
}

.fuzzy-words-list {
  display: grid;
  gap: 6px;
}

.fuzzy-word-row {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 8px;
  align-items: center;
  padding: 8px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.8);
  cursor: pointer;
}

.fuzzy-word-row.is-active {
  border-color: rgba(27, 154, 170, 0.4);
  box-shadow: 0 10px 16px -16px rgba(27, 154, 170, 0.5);
}

.fuzzy-word-checkbox input {
  width: 12px;
  height: 12px;
}

.fuzzy-word-meta {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.fuzzy-word-text {
  font-size: 0.6rem;
  font-weight: 600;
}

.fuzzy-word-meaning {
  font-size: 0.5rem;
  color: var(--muted);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.fuzzy-word-action {
  padding: 4px 10px;
  border-radius: 8px;
  border: 1px solid rgba(180, 35, 24, 0.35);
  background: rgba(255, 237, 235, 0.8);
  font-size: 0.5rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  color: #b42318;
  cursor: pointer;
}

.fuzzy-detail-page {
  display: grid;
  grid-template-rows: auto 1fr;
  gap: 8px;
  min-height: 100%;
  height: 100%;
}

.fuzzy-detail-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.fuzzy-detail-word {
  font-size: 0.95rem;
  font-weight: 600;
  font-family: "Fraunces", serif;
}

.fuzzy-detail-body {
  display: grid;
  gap: 8px;
  padding: 10px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.7);
}

.fuzzy-detail-meaning {
  margin: 0;
  font-size: 0.6rem;
  line-height: 1.3;
  color: #2a2723;
}

.fuzzy-detail-audio {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.fuzzy-audio-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 999px;
  border: 1px solid rgba(31, 29, 26, 0.08);
  background: #f4f5f7;
  font-size: 0.5rem;
  font-weight: 600;
  color: #1f1d1a;
  cursor: pointer;
}

.fuzzy-audio-pill:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.fuzzy-audio-pill.is-playing {
  border-color: rgba(255, 75, 75, 0.35);
  box-shadow: 0 6px 12px -10px rgba(255, 75, 75, 0.6);
}

.fuzzy-audio-pill.is-playing .fuzzy-audio-icon {
  animation: fuzzy-audio-pulse 0.9s ease-in-out infinite;
}

.fuzzy-audio-pill.is-playing .fuzzy-audio-phonetic {
  color: #1f1d1a;
}

.fuzzy-audio-label {
  font-size: 0.5rem;
  font-weight: 600;
}

.fuzzy-audio-phonetic {
  font-size: 0.5rem;
  color: #7a7a7a;
  font-weight: 500;
  letter-spacing: 0.02em;
}

.fuzzy-audio-icon {
  font-size: 0.55rem;
  color: #ff4b4b;
}

@keyframes fuzzy-audio-pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.6;
  }
  50% {
    transform: scale(1.2);
    opacity: 1;
  }
}

.fuzzy-audio-empty {
  margin: 0;
  font-size: 0.5rem;
  color: var(--muted);
}

.fuzzy-detail-examples {
  display: grid;
  gap: 2px;
}

.dictionary-download {
  display: grid;
  gap: 8px;
  align-content: start;
  min-height: 100%;
}

.dictionary-download-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.dictionary-download-title {
  font-size: 0.6rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.dictionary-download-desc {
  margin: 0;
  font-size: 0.52rem;
  color: var(--muted);
}

.dictionary-download-notice {
  margin: 0;
  font-size: 0.52rem;
  color: #9b1c1c;
}

.dictionary-download-list {
  display: grid;
  gap: 6px;
}

.dictionary-download-item {
  padding: 8px;
  border-radius: 10px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.85);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.dictionary-download-name {
  font-size: 0.6rem;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.dictionary-download-meta {
  display: grid;
  gap: 2px;
}

.dictionary-download-state {
  font-size: 0.5rem;
  color: var(--muted);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.dictionary-download-state.is-downloaded {
  color: #1b9aaa;
  font-weight: 600;
}

.dictionary-download-error {
  font-size: 0.5rem;
  color: #b42318;
}

.dictionary-download-action {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: rgba(255, 255, 255, 0.9);
  font-size: 0.52rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  cursor: pointer;
  color: #1f1d1a;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.dictionary-download-action:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 14px -14px var(--shadow);
}

.dictionary-download-action:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}
.settings-error {
  color: #9b1c1c;
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
  .icon-button,
  .dictionary-download-action {
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






