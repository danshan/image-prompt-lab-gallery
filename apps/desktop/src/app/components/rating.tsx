import React from "react";

export function StarRatingDisplay({ rating, showEmpty = false }: { rating: number | null; showEmpty?: boolean }) {
  const value = rating ?? 0;
  if (value === 0 && !showEmpty) {
    return <span className="star-rating empty" aria-label="Not rated">Unrated</span>;
  }
  return (
    <span className="star-rating" aria-label={`${value} of 5 stars`}>
      {[1, 2, 3, 4, 5].map((star) => (
        <span key={star} className={star <= value ? "star filled" : "star"}>
          {star <= value ? "★" : "☆"}
        </span>
      ))}
    </span>
  );
}

export function StarRatingControl({
  rating,
  onChange,
}: {
  rating: number | null;
  onChange: (rating: number) => void;
}) {
  const value = rating ?? 0;
  return (
    <div className="star-rating-control" role="radiogroup" aria-label="Rating">
      {[1, 2, 3, 4, 5].map((star) => (
        <button
          key={star}
          className={star <= value ? "star-button active" : "star-button"}
          aria-label={`${star} star${star === 1 ? "" : "s"}`}
          aria-checked={value === star}
          role="radio"
          onClick={() => onChange(star)}
        >
          {star <= value ? "★" : "☆"}
        </button>
      ))}
    </div>
  );
}

