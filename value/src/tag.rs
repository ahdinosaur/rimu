use rimu_meta::{Span, Spanned};

use crate::{BothTagged, EvalError, SpannedValue, Value, ValueMeta};

pub type TagMeta = (String, ValueMeta);

/// Strips an outer [`Value::Tagged`] wrapper if present, returning the inner
/// value and the tag + meta for later re-wrapping.
pub fn peel_tag(value: Value) -> (Value, Option<TagMeta>) {
    match value {
        Value::Tagged { tag, inner, meta } => (inner.into_inner(), Some((tag, meta))),
        other => (other, None),
    }
}

/// Combines two optional tag+meta pairs.
///
/// - Same tag: metas are merged (right-wins on key collision).
/// - Different tags: error via [`BothTagged`].
/// - One side `None`: the other passes through.
pub fn merge_tag_metas(
    left: Option<TagMeta>,
    left_span: Span,
    right: Option<TagMeta>,
    right_span: Span,
) -> Result<Option<TagMeta>, EvalError> {
    match (left, right) {
        (None, None) => Ok(None),
        (Some(lt), None) => Ok(Some(lt)),
        (None, Some(rt)) => Ok(Some(rt)),
        (Some((left_tag, mut left_meta)), Some((right_tag, right_meta))) => {
            if left_tag != right_tag {
                return Err(EvalError::BothTagged(Box::new(BothTagged {
                    left_span,
                    right_span,
                    left_tag,
                    right_tag,
                })));
            }
            for (key, value) in right_meta {
                left_meta.insert(key, value);
            }
            Ok(Some((left_tag, left_meta)))
        }
    }
}

/// Re-wraps a computed inner value with a tag + meta, if any. The outer
/// [`Spanned`] uses `span`; the inner [`SpannedValue`] keeps whatever span
/// it came in with.
pub fn rewrap_tag(span: Span, inner: SpannedValue, tag_meta: Option<TagMeta>) -> SpannedValue {
    match tag_meta {
        None => inner,
        Some((tag, meta)) => Spanned::new(
            Value::Tagged {
                tag,
                inner: Box::new(inner),
                meta,
            },
            span,
        ),
    }
}
