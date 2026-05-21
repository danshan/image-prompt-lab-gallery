import { clearSelectionForLibrarySwitch, type DetailLoadState } from "../gallery/state.js";
import { type ReviewFormState } from "../review/state.js";

export type LibraryWorkspaceClearState<TDetail> = {
  selectedAssetId: string;
  selectedGalleryAssetIds: string[];
  detailState: DetailLoadState<TDetail>;
  selectedAlbumId: string | null;
  selectedSuggestionId: string | null;
  selectedSuggestionIds: string[];
  reviewForm: ReviewFormState | null;
  selectedTaskId: string | null;
};

export function clearLibraryWorkspaceState<TDetail>(): LibraryWorkspaceClearState<TDetail> {
  return {
    selectedAssetId: "",
    selectedGalleryAssetIds: [],
    detailState: clearSelectionForLibrarySwitch<TDetail>(),
    selectedAlbumId: null,
    selectedSuggestionId: null,
    selectedSuggestionIds: [],
    reviewForm: null,
    selectedTaskId: null,
  };
}
