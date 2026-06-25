import type { DragEvent } from "react";
import { Fragment } from "react";
import lumoraLogo from "../assets/lumora-logo-256.png";
import type { DockItem } from "../lib/dockItems";
import { filePathToAssetSrc } from "../lib/native";

type DockSurfaceProps = {
  dockItems: DockItem[];
  isDropHot: boolean;
  onDragStateChange: (isHot: boolean) => void;
  onDrop: (event: DragEvent<HTMLElement>) => void;
  onOpen: (item: DockItem) => void;
};

export function DockSurface({ dockItems, isDropHot, onDragStateChange, onDrop, onOpen }: DockSurfaceProps) {
  return (
    <main className="dock-window-surface">
      <nav
        className={`lumora-dock ${isDropHot ? "is-drop-hot" : ""}`}
        aria-label="Lumora Dock"
        onDragEnter={() => onDragStateChange(true)}
        onDragOver={(event) => event.preventDefault()}
        onDragLeave={() => onDragStateChange(false)}
        onDrop={onDrop}
      >
        {dockItems.map((item) => (
          <Fragment key={item.id}>
            {item.id === "trash" ? <span className="dock-separator" aria-hidden="true" /> : null}
            <button
              className={`dock-icon dock-${item.tone} ${item.id === "trash" ? "dock-trash-icon" : ""}`}
              aria-label={item.type === "launcher" ? "光枢" : item.label}
              title={item.label}
              onClick={() => onOpen(item)}
            >
              {item.type === "launcher" ? (
                <img className="dock-logo" src={lumoraLogo} alt="" aria-hidden="true" />
              ) : item.id === "trash" ? (
                <span className="dock-trash-can" aria-hidden="true">
                  <span />
                </span>
              ) : item.iconPath ? (
                <img className="dock-app-icon" src={filePathToAssetSrc(item.iconPath)} alt="" aria-hidden="true" />
              ) : (
                <span>{item.glyph}</span>
              )}
              {item.active ? <i aria-hidden="true" /> : null}
            </button>
          </Fragment>
        ))}
      </nav>
    </main>
  );
}
