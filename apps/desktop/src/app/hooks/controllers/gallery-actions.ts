import type { Dispatch, SetStateAction } from "react";
import {
  beginDetailLoad,
  completeDetailLoad,
  failDetailLoad,
  type DetailLoadState,
  type GalleryQueryState,
} from "../../workflows/gallery";
import {
  mockDetailFor,
} from "../../utils";
import { galleryQueryInput } from "../../workflows/gallery/query";
import {
  errorMessage,
  invokeCommand,
} from "../../tauri-adapter";
import type {
  AssetDetail,
  GalleryAsset,
  Library,
} from "../../types";

export function useGallerySelectionActions({
  library,
  query,
  detailState,
  setGallery,
  setSelectedAssetId,
  setDetailState,
  setInspectorOpen,
  setStatus,
  setRecoverableError,
}: {
  library: Library | null;
  query: GalleryQueryState;
  detailState: DetailLoadState<AssetDetail>;
  setGallery: Dispatch<SetStateAction<GalleryAsset[]>>;
  setSelectedAssetId: Dispatch<SetStateAction<string>>;
  setDetailState: Dispatch<SetStateAction<DetailLoadState<AssetDetail>>>;
  setInspectorOpen: Dispatch<SetStateAction<boolean>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
}) {
  function selectGalleryAsset(assetId: string) {
    setSelectedAssetId(assetId);
    setInspectorOpen(true);
  }

  function selectAssetVersion(versionId: string) {
    const detail = detailState.detail;
    if (!detail) {
      return;
    }
    void loadAssetDetail(detail.id, versionId);
  }

  async function refreshGallery() {
    if (!library) {
      return;
    }
    try {
      const items = await invokeCommand<GalleryAsset[]>("query_gallery", {
        input: galleryQueryInput(library.rootPath, query),
      });
      setGallery(items);
      setSelectedAssetId((current) => items.find((item) => item.id === current)?.id ?? items[0]?.id ?? "");
      setStatus(`${items.length} item${items.length === 1 ? "" : "s"}`);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function loadAssetDetail(assetId: string, versionId: string | null) {
    if (!library) {
      return;
    }
    setDetailState(beginDetailLoad(assetId));
    try {
      const detail = await invokeCommand<AssetDetail>("get_asset_detail", {
        input: {
          libraryPath: library.rootPath,
          assetId,
          currentVersionId: versionId,
        },
      });
      setDetailState(completeDetailLoad(assetId, detail));
    } catch (error) {
      setDetailState(failDetailLoad(assetId, errorMessage(error)));
    }
  }

  function loadPreviewAssetDetail(asset: GalleryAsset) {
    setDetailState(completeDetailLoad(asset.id, mockDetailFor(asset)));
  }

  async function archiveAssets(assetIds: string[]) {
    if (!library || assetIds.length === 0) {
      return;
    }
    try {
      for (const assetId of assetIds) {
        await invokeCommand<void>("archive_asset", {
          input: {
            libraryPath: library.rootPath,
            assetId,
          },
        });
      }
      setGallery((items) => items.filter((item) => !assetIds.includes(item.id)));
      setSelectedAssetId((current) => (assetIds.includes(current) ? "" : current));
      setStatus(`${assetIds.length} archived`);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  return {
    selectGalleryAsset,
    selectAssetVersion,
    refreshGallery,
    loadAssetDetail,
    loadPreviewAssetDetail,
    archiveAssets,
  };
}
