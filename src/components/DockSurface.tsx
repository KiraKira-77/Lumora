import type { DragEvent } from "react";
import { Fragment, useState, useEffect, useRef } from "react";
import lumoraLogo from "../assets/lumora-logo-256.png";
import trashGlassmorphism from "../assets/trash-glassmorphism.png";
import type { DockItem, NewDockItemInput } from "../lib/dockItems";
import { filePathToAssetSrc, updateDockWindowBounds } from "../lib/native";

type DockSurfaceProps = {
  dockItems: DockItem[];
  dropPreviewItems?: NewDockItemInput[];
  isDropHot: boolean;
  onDragStateChange: (isHot: boolean) => void;
  onDrop: (event: DragEvent<HTMLElement>) => void;
  onOpen: (item: DockItem) => void;
  onRemove: (item: DockItem) => void;
};

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
  dropPreviewItems = [],
  isDropHot,
  onDragStateChange,
  onDrop,
  onOpen,
  onRemove,
}: DockSurfaceProps) {
  const [contextMenu, setContextMenu] = useState<{ id: string; x: number; y: number } | null>(null);
  const navRef = useRef<HTMLElement>(null);

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

  return (
    <main className="dock-window-surface">
      <nav
        ref={navRef}
        className={`lumora-dock ${isDropHot ? "is-drop-hot" : ""}`}
        aria-label="Lumora Dock"
        onDragEnter={() => onDragStateChange(true)}
        onDragOver={(event) => event.preventDefault()}
        onDragLeave={() => onDragStateChange(false)}
        onDrop={onDrop}
      >
        {dockItems.map((item) => (
          <Fragment key={item.id}>
            {item.id === "trash" ? (
              <>
                {dropPreviewItems.map((preview) => {
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
                })}
                <span className="dock-separator" aria-hidden="true" />
              </>
            ) : null}
            <button
              className={`dock-icon ${item.iconPath && item.type !== "launcher" ? "dock-transparent" : item.id === "trash" ? "dock-transparent dock-trash" : `dock-${item.tone}`}`}
              aria-label={item.type === "launcher" ? "光枢" : item.label}
              title={item.label}
              onClick={() => onOpen(item)}
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
              {item.active ? <i aria-hidden="true" /> : null}
              
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
        ))}
      </nav>
    </main>
  );
}
