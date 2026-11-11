//! `Accordion` and its parts, with the story's FAQ. The Default story's play function opens
//! Shipping, then Returns, which closes Shipping, so the root shows Returns open. Penpot's
//! main component holds three items with slot placeholders for their bodies.

use brrritscold_core_gpui::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use brrritscold_core_gpui_tokens::primitive::size;
use gpui::{IntoElement as _, ParentElement as _, Styled as _};

use super::{Entry, one};

const ITEMS: [(&str, &str, &str); 3] = [
    (
        "shipping",
        "What are your shipping options?",
        "Standard shipping takes three to five days; express arrives the next working day.",
    ),
    (
        "returns",
        "What is your return policy?",
        "Return any unused item within 30 days for a full refund.",
    ),
    (
        "support",
        "How do I reach support?",
        "Write to us any time; we answer within one working day.",
    ),
];

fn item((value, question, answer): (&'static str, &'static str, &'static str)) -> AccordionItem {
    AccordionItem::new(value)
        .trigger(AccordionTrigger::new(question))
        .content(AccordionContent::new().child(answer))
}

pub(super) fn entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "Accordion",
            properties: &[],
            // The story's max-w-md.
            variants: || {
                one(Some("components-accordion--default"), || {
                    ITEMS
                        .into_iter()
                        .fold(Accordion::new("faq"), |accordion, faq| {
                            accordion.item(item(faq))
                        })
                        .default_value(["returns"])
                        .w(size::MD)
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "AccordionItem",
            properties: &[],
            // An item on its own is an accordion of one: open, as Penpot's main component is.
            variants: || {
                one(None, || {
                    Accordion::new("item")
                        .item(item(ITEMS[0]))
                        .default_value(["shipping"])
                        .w(size::MD)
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "AccordionTrigger",
            properties: &[],
            variants: || {
                one(None, || {
                    AccordionTrigger::new(ITEMS[0].1)
                        .w(size::MD)
                        .into_any_element()
                })
            },
        },
        Entry {
            name: "AccordionContent",
            properties: &[],
            variants: || {
                one(None, || {
                    AccordionContent::new()
                        .w(size::MD)
                        .child(ITEMS[0].2)
                        .into_any_element()
                })
            },
        },
    ]
}
