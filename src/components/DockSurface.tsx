import type { DragEvent } from "react";
import { Fragment, useCallback, useState, useEffect, useRef } from "react";
import lumoraLogo from "../assets/lumora-logo-256.png";
import trashGlassmorphism from "../assets/trash-glassmorphism.png";
import type { DockItem, NewDockItemInput } from "../lib/dockItems";
import { dockInsertIndexFromPointer } from "../lib/dockDropPosition";
import { filePathToAssetSrc, updateDockWindowBounds } from "../lib/native";

type DockSurfaceProps = {
  dockItems: DockItem[];
  dockItemStatuses?: Record<string, DockRuntimeStatus>;
  dropPreviewItems?: NewDockItemInput[];
  isDropHot: boolean;
  onDragStateChange: (isHot: boolean) => void;
  onDrop: (event: DragEvent<HTMLElement>, insertIndex: number) => void;
  onOpen: (item: DockItem) => void;
  onRemove: (item: DockItem) => void;
  onInsertIndexResolverChange?: (resolver: DockInsertIndexResolver | null) => void;
  onReorder?: (draggedId: string, targetId: string) => void;
};

export type DockRuntimeStatus = {
  isRunning: boolean;
  needsAttention: boolean;
  attentionSequence: number;
};

export type DockInsertIndexResolver = (pointerX: number) => number;

const dockDragType = "application/x-lumora-dock-item";

function previewTone(type: NewDockItemInput["type"]): string {
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
  }
}

export function DockSurface({
  dockItems,
  dockItemStatuses = {},
  dropPreviewItems = [],
  isDropHot,
  onDragStateChange,
  onDrop,
  onOpen,
  onRemove,
  onInsertIndexResolverChange,
  onReorder,
}: DockSurfaceProps) {
  const [contextMenu, setContextMenu] = useState<{ id: string; x: number; y: number } | null>(null);
  const navRef = useRef<HTMLElement>(null);
  const draggedDockItemIdRef = useRef<string | null>(null);
  const suppressOpenAfterDragRef = useRef(false);
  const [dropInsertIndex, setDropInsertIndex] = useState<number | null>(null);

  useEffect(() => {
    const closeMenu = () => setContextMenu(null);
    window.addEventListener("click", closeMenu);
    return () => window.removeEventListener("click", closeMenu);
  }, []);

  useEffect(() => {
    if (!navRef.current) return;
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const width = navRef.current!.offsetWidth;
        const height = navRef.current!.offsetHeight;
        updateDockWindowBounds(width, height).catch(console.error);
      }
    });
    observer.observe(navRef.current);
    return () => observer.disconnect();
  }, []);

  function isInternalDockDrag(event: DragEvent<HTMLElement>): boolean {
    return draggedDockItemIdRef.current !== null || Array.from(event.dataTransfer.types).includes(dockDragType);
  }

  function handleDockItemDragStart(item: DockItem, event: DragEvent<HTMLButtonElement>) {
    if (item.pinned) {
      event.preventDefault();
      return;
    }

    draggedDockItemIdRef.current = item.id;
    suppressOpenAfterDragRef.current = true;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(dockDragType, item.id);
  }

  function handleDockItemDragOver(item: DockItem, event: DragEvent<HTMLButtonElement>) {
    if (item.pinned || !isInternalDockDrag(event)) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    event.dataTransfer.dropEffect = "move";
  }

  function handleDockItemDrop(item: DockItem, event: DragEvent<HTMLButtonElement>) {
    if (!isInternalDockDrag(event)) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    const draggedId = event.dataTransfer.getData(dockDragType) || draggedDockItemIdRef.current;
    draggedDockItemIdRef.current = null;

    if (!draggedId || draggedId === item.id || item.pinned) {
      return;
    }

    onReorder?.(draggedId, item.id);
  }

  function handleDockItemDragEnd() {
    draggedDockItemIdRef.current = null;
    window.setTimeout(() => {
      suppressOpenAfterDragRef.current = false;
    }, 0);
  }

  const defaultInsertIndex = useCallback((): number => {
    const trashIndex = dockItems.findIndex((item) => item.id === "trash");
    return trashIndex >= 0 ? trashIndex : dockItems.length;
  }, [dockItems]);

  const insertIndexFromPointer = useCallback((pointerX: number): number => {
    const buttons = Array.from(navRef.current?.querySelectorAll<HTMLElement>("[data-dock-item-id]") ?? []);
    if (buttons.length !== dockItems.length) {
      return defaultInsertIndex();
    }

    return dockInsertIndexFromPointer(
      dockItems.map((item, index) => {
        const rect = buttons[index].getBoundingClientRect();
        return {
          id: item.id,
          pinned: item.pinned,
          left: rect.left,
          right: rect.right,
        };
      }),
      pointerX,
    );
  }, [defaultInsertIndex, dockItems]);

  const updateDropInsertIndex = useCallback((pointerX: number) => {
    const nextIndex = insertIndexFromPointer(pointerX);
    setDropInsertIndex(nextIndex);
    return nextIndex;
  }, [insertIndexFromPointer]);

  useEffect(() => {
    onInsertIndexResolverChange?.(updateDropInsertIndex);
    return () => onInsertIndexResolverChange?.(null);
  }, [onInsertIndexResolverChange, updateDropInsertIndex]);

  useEffect(() => {
    if (!isDropHot) {
      setDropInsertIndex(null);
    }
  }, [isDropHot]);

  function renderDropPreviewItems() {
    return dropPreviewItems.map((preview) => {
      const previewToneClass = preview.iconPath ? "dock-transparent" : `dock-${previewTone(preview.type)}`;
      return (
        <button
          className={`dock-icon ${previewToneClass} dock-drop-preview`}
          aria-label={`即将添加 ${preview.label}`}
          title={`即将添加 ${preview.label}`}
          key={`${preview.target}-${preview.label}`}
          type="button"
        >
          {preview.iconPath ? (
            <img className="dock-app-icon" src={filePathToAssetSrc(preview.iconPath)} alt="" aria-hidden="true" />
          ) : (
            <span>{preview.label.slice(0, 1).toUpperCase() || "?"}</span>
          )}
        </button>
      );
    });
  }

  const previewInsertIndex = dropPreviewItems.length > 0 ? (dropInsertIndex ?? defaultInsertIndex()) : -1;

  return (
    <main className="dock-window-surface">
      <nav
        ref={navRef}
        className={`lumora-dock ${isDropHot ? "is-drop-hot" : ""}`}
        aria-label="Lumora Dock"
        onDragEnter={(event) => {
          if (isInternalDockDrag(event)) {
            return;
          }
          updateDropInsertIndex(event.clientX);
          onDragStateChange(true);
        }}
        onDragOver={(event) => {
          event.preventDefault();
          if (!isInternalDockDrag(event)) {
            updateDropInsertIndex(event.clientX);
          }
        }}
        onDragLeave={(event) => {
          if (isInternalDockDrag(event)) {
            return;
          }
          onDragStateChange(false);
          setDropInsertIndex(null);
        }}
        onDrop={(event) => {
          if (isInternalDockDrag(event)) {
            event.preventDefault();
            draggedDockItemIdRef.current = null;
            return;
          }
          const insertIndex = updateDropInsertIndex(event.clientX);
          onDrop(event, insertIndex);
          setDropInsertIndex(null);
        }}
      >
        {dockItems.map((item, index) => {
          const status = dockItemStatuses[item.target];
          const attentionSequence = status?.attentionSequence ?? 0;
          const isAttention = Boolean(status?.needsAttention);
          const bounceClass =
            isAttention && attentionSequence > 0
              ? ` is-bouncing bounce-${attentionSequence % 2 === 0 ? "even" : "odd"}`
              : "";
          const itemClassName = `dock-icon ${
            item.iconPath && item.type !== "launcher"
              ? "dock-transparent"
              : item.id === "trash"
                ? "dock-transparent dock-trash"
                : `dock-${item.tone}`
          }${isAttention ? " is-attention" : ""}${bounceClass}`;

          return (
          <Fragment key={item.id}>
            {index === previewInsertIndex ? renderDropPreviewItems() : null}
            {item.id === "trash" ? (
              <>
                <span className="dock-separator" aria-hidden="true" />
              </>
            ) : null}
            <button
              className={itemClassName}
              aria-label={item.type === "launcher" ? "光枢" : item.label}
              title={item.label}
              data-dock-item-id={item.id}
              draggable={!item.pinned}
              onDragStart={(event) => handleDockItemDragStart(item, event)}
              onDragEnter={(event) => {
                if (isInternalDockDrag(event)) {
                  event.stopPropagation();
                }
              }}
              onDragOver={(event) => handleDockItemDragOver(item, event)}
              onDrop={(event) => handleDockItemDrop(item, event)}
              onDragEnd={handleDockItemDragEnd}
              onClick={(event) => {
                if (suppressOpenAfterDragRef.current) {
                  event.preventDefault();
                  return;
                }
                onOpen(item);
              }}
              onContextMenu={(e) => {
                e.preventDefault();
                if (!item.pinned) {
                  setContextMenu({ id: item.id, x: 0, y: 0 }); // Just using id now
                }
              }}
            >
              {item.type === "launcher" ? (
                <img className="dock-logo" src={lumoraLogo} alt="" aria-hidden="true" />
              ) : item.id === "trash" ? (
                <img src={trashGlassmorphism} alt="回收站" aria-hidden="true" style={{ width: "46px", height: "46px", objectFit: "contain", filter: "drop-shadow(0 4px 6px rgba(0,0,0,0.15))", transform: "translateY(-2px)" }} />
              ) : item.iconPath ? (
                <img className="dock-app-icon" src={filePathToAssetSrc(item.iconPath)} alt="" aria-hidden="true" />
              ) : (
                <span>{item.glyph}</span>
              )}
              {(status?.isRunning || item.active || isAttention) ? <span className="dock-running-indicator" aria-hidden="true" /> : null}
              
              {contextMenu && contextMenu.id === item.id && (
                <div 
                  className="dock-context-menu" 
                  style={{ 
                    position: "absolute", 
                    top: "50%",
                    left: "50%",
                    transform: "translate(-50%, -50%)",
                    background: "linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(245, 245, 245, 0.8))",
                    border: "1px solid rgba(255, 255, 255, 0.6)",
                    borderRadius: "12px",
                    padding: "4px",
                    zIndex: 9999,
                    backdropFilter: "blur(20px) saturate(1.2)",
                    boxShadow: "0 4px 12px rgba(0, 0, 0, 0.1), 0 0 0 1px rgba(255, 255, 255, 0.3)"
                  }}
                  onClick={(e) => e.stopPropagation()} // Prevent triggering onOpen
                >
                  <button 
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemove(item);
                      setContextMenu(null);
                    }}
                    style={{
                      background: "none",
                      border: "none",
                      color: "#e63946",
                      padding: "6px 14px",
                      cursor: "pointer",
                      fontSize: "13px",
                      fontWeight: "600",
                      borderRadius: "8px",
                      fontFamily: "inherit",
                      whiteSpace: "nowrap",
                      outline: "none"
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.backgroundColor = "rgba(230, 57, 70, 0.1)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.backgroundColor = "transparent";
                    }}
                  >
                    移除
                  </button>
                </div>
              )}
            </button>
          </Fragment>
          );
        })}
      </nav>
    </main>
  );
}
