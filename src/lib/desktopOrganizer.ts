export type DesktopCategory =
  | "Images"
  | "Docs"
  | "Archives"
  | "Installers"
  | "Videos"
  | "Projects"
  | "Inbox";

export type DesktopFileLike = {
  name: string;
  path: string;
  category: DesktopCategory;
  label: string;
};

export type OrganizerPlanGroup = {
  category: DesktopCategory;
  label: string;
  count: number;
  files: DesktopFileLike[];
};

export type OrganizerPlan = {
  rootFolderName: string;
  totalFiles: number;
  groups: OrganizerPlanGroup[];
};

const categoryByExtension: Record<string, DesktopCategory> = {
  ".png": "Images",
  ".jpg": "Images",
  ".jpeg": "Images",
  ".webp": "Images",
  ".gif": "Images",
  ".svg": "Images",
  ".pdf": "Docs",
  ".doc": "Docs",
  ".docx": "Docs",
  ".xls": "Docs",
  ".xlsx": "Docs",
  ".ppt": "Docs",
  ".pptx": "Docs",
  ".txt": "Docs",
  ".md": "Docs",
  ".zip": "Archives",
  ".rar": "Archives",
  ".7z": "Archives",
  ".tar": "Archives",
  ".gz": "Archives",
  ".exe": "Installers",
  ".msi": "Installers",
  ".mp4": "Videos",
  ".mov": "Videos",
  ".mkv": "Videos",
  ".avi": "Videos",
  ".sln": "Projects",
  ".csproj": "Projects",
  ".package": "Projects",
  ".json": "Projects",
  ".ts": "Projects",
  ".tsx": "Projects",
  ".rs": "Projects",
};

export function classifyDesktopFile(fileName: string): DesktopCategory {
  const dotIndex = fileName.lastIndexOf(".");

  if (dotIndex <= 0 || dotIndex === fileName.length - 1) {
    return "Inbox";
  }

  const extension = fileName.slice(dotIndex).toLowerCase();
  return categoryByExtension[extension] ?? "Inbox";
}

export function buildOrganizerPlan(files: DesktopFileLike[]): OrganizerPlan {
  const groups = files.reduce<OrganizerPlanGroup[]>((nextGroups, file) => {
    const existingGroup = nextGroups.find((group) => group.category === file.category);
    if (existingGroup) {
      existingGroup.count += 1;
      existingGroup.files.push(file);
      return nextGroups;
    }

    nextGroups.push({
      category: file.category,
      label: file.label,
      count: 1,
      files: [file],
    });
    return nextGroups;
  }, []);

  return {
    rootFolderName: "Lumora整理",
    totalFiles: files.length,
    groups,
  };
}
