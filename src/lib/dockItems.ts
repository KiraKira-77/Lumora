export type DockItemType = "launcher" | "app" | "folder" | "file" | "url" | "action" | "settings";

export type DockItem = {
  id: string;
  label: string;
  type: DockItemType;
  target: string;
  glyph: string;
  tone: string;
  iconPath?: string;
  order: number;
  pinned: boolean;
  active: boolean;
};

export type NewDockItemInput = {
  label: string;
  type: Exclude<DockItemType, "launcher">;
  target: string;
  iconPath?: string;
};

export type DroppedTargetKind = "app" | "folder" | "file" | "url";

const defaultItems: DockItem[] = [
  {
    id: "launcher",
    label: "光枢",
    type: "launcher",
    target: "lumora://launcher",
    glyph: "L",
    tone: "launcher",
    order: 0,
    pinned: true,
    active: true,
  },
  {
    id: "trash",
    label: "垃圾桶",
    type: "action",
    target: "lumora://trash",
    glyph: "T",
    tone: "trash",
    order: 1,
    pinned: true,
    active: false,
  },
];

export function createDefaultDockItems(): DockItem[] {
  return defaultItems.map((item) => ({ ...item }));
}

export function addDockItem(items: DockItem[], input: NewDockItemInput): DockItem[] {
  const cleanLabel = input.label.trim();
  const cleanTarget = input.target.trim();
  const sorted = sortDockItems(items);
  const trashIndex = sorted.findIndex((item) => item.id === "trash");
  const insertIndex = trashIndex >= 0 ? trashIndex : sorted.length;

  const next = [
    ...sorted.slice(0, insertIndex),
    {
      id: `dock_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
      label: cleanLabel,
      type: input.type,
      target: cleanTarget,
      glyph: glyphForLabel(cleanLabel),
      tone: toneForType(input.type),
      iconPath: input.iconPath,
      order: insertIndex,
      pinned: false,
      active: false,
    },
    ...sorted.slice(insertIndex),
  ];

  return normalizeDockOrder(next);
}

export function createDockItemInputFromTarget(target: string, kind?: DroppedTargetKind): NewDockItemInput {
  const cleanTarget = target.trim();
  const type = kind ?? inferTargetType(cleanTarget);

  return {
    label: labelForTarget(cleanTarget, type),
    type,
    target: cleanTarget,
  };
}

export function searchDockItems(items: DockItem[], query: string): DockItem[] {
  const normalizedQuery = query.trim().toLowerCase();

  if (!normalizedQuery) {
    return sortDockItems(items);
  }

  return sortDockItems(items).filter((item) => {
    const haystack = `${item.label} ${item.type} ${item.target}`.toLowerCase();
    return haystack.includes(normalizedQuery);
  });
}

export function sortDockItems(items: DockItem[]): DockItem[] {
  return [...items].sort((a, b) => a.order - b.order);
}

export function moveDockItem(items: DockItem[], id: string, direction: -1 | 1): DockItem[] {
  const sorted = sortDockItems(items);
  const index = sorted.findIndex((item) => item.id === id);
  const nextIndex = index + direction;

  if (index < 0 || nextIndex < 0 || nextIndex >= sorted.length || sorted[index].pinned) {
    return sorted;
  }

  const target = sorted[nextIndex];
  if (target.pinned) {
    return sorted;
  }

  [sorted[index], sorted[nextIndex]] = [sorted[nextIndex], sorted[index]];
  return normalizeDockOrder(sorted);
}

export function removeDockItem(items: DockItem[], id: string): DockItem[] {
  return normalizeDockOrder(sortDockItems(items).filter((item) => item.id !== id || item.pinned));
}

function normalizeDockOrder(items: DockItem[]): DockItem[] {
  return items.map((item, order) => ({ ...item, order }));
}

function glyphForLabel(label: string): string {
  return label.slice(0, 1).toUpperCase() || "?";
}

function toneForType(type: DockItemType): string {
  switch (type) {
    case "app":
      return "code";
    case "folder":
      return "folder";
    case "url":
      return "chrome";
    case "action":
      return "organize";
    case "settings":
      return "settings";
    case "file":
      return "shot";
    case "launcher":
      return "launcher";
  }
}

function inferTargetType(target: string): DroppedTargetKind {
  if (isUrl(target)) {
    return "url";
  }

  const extension = extensionForTarget(target);
  if ([".exe", ".lnk", ".bat", ".cmd", ".ps1"].includes(extension)) {
    return "app";
  }

  if (extension) {
    return "file";
  }

  return "folder";
}

function labelForTarget(target: string, type: DroppedTargetKind): string {
  if (type === "url") {
    try {
      const url = new URL(target);
      return url.hostname.replace(/^www\./, "") || target;
    } catch {
      return target;
    }
  }

  const name = basename(target);
  if (!name) {
    return target;
  }

  if (type === "folder") {
    return name;
  }

  const extension = extensionForTarget(name);
  return extension ? name.slice(0, -extension.length) : name;
}

function basename(target: string): string {
  const cleanTarget = target.replace(/[\\/]+$/, "");
  return cleanTarget.split(/[\\/]/).pop() ?? cleanTarget;
}

function extensionForTarget(target: string): string {
  const name = basename(target);
  const dotIndex = name.lastIndexOf(".");

  if (dotIndex <= 0 || dotIndex === name.length - 1) {
    return "";
  }

  return name.slice(dotIndex).toLowerCase();
}

function isUrl(target: string): boolean {
  try {
    const url = new URL(target);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}
