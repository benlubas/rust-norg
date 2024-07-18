use serde::Serialize;

use chumsky::prelude::*;
use chumsky::Parser;

use crate::ParagraphSegmentToken;
use crate::{
    stage_2::ParagraphSegment, stage_3::NorgASTFlat, CarryoverTag, DetachedModifierExtension,
    NestableDetachedModifier, RangeableDetachedModifier,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub enum NorgAST {
    Paragraph(ParagraphSegment),
    NestableDetachedModifier {
        modifier_type: NestableDetachedModifier,
        level: u16,
        extensions: Vec<DetachedModifierExtension>,
        content: Box<NorgASTFlat>,
    },
    RangeableDetachedModifier {
        modifier_type: RangeableDetachedModifier,
        title: ParagraphSegment,
        extensions: Vec<DetachedModifierExtension>,
        content: Vec<NorgASTFlat>,
    },
    Heading {
        level: u16,
        title: ParagraphSegment,
        extensions: Vec<DetachedModifierExtension>,
        content: Vec<Self>,
    },
    CarryoverTag {
        tag_type: CarryoverTag,
        name: Vec<String>,
        parameters: Vec<String>,
        next_object: Box<NorgASTFlat>,
    },
    VerbatimRangedTag {
        name: Vec<String>,
        parameters: Vec<String>,
        content: String,
    },
    RangedTag {
        name: Vec<String>,
        parameters: Vec<String>,
        content: Vec<NorgASTFlat>,
    },
    InfirmTag {
        name: Vec<String>,
        parameters: Vec<String>,
    },
}

/// Parse a heading of the given level, recursively parsing it's content as well.
pub fn heading_lvl(
    current_level: u16,
    title: Vec<ParagraphSegmentToken>,
    extensions: Vec<DetachedModifierExtension>,
) -> impl Parser<NorgASTFlat, Vec<NorgAST>, Error = chumsky::error::Simple<NorgASTFlat>> {
    let content = choice((
        select! { NorgASTFlat::Paragraph(segment) => NorgAST::Paragraph(segment) },
        // TODO: pull these out into their own thing
        select! {
            NorgASTFlat::NestableDetachedModifier { modifier_type, level, extensions, content } =>
                NorgAST::NestableDetachedModifier { modifier_type, level, extensions, content }
        },
        select! {
            NorgASTFlat::RangeableDetachedModifier { modifier_type, title, extensions, content } =>
                NorgAST::RangeableDetachedModifier { modifier_type, title, extensions, content }
        },
        select! {
            NorgASTFlat::CarryoverTag { tag_type, name, parameters, next_object } =>
                NorgAST::CarryoverTag { tag_type, name, parameters, next_object }
        },
        select! { NorgASTFlat::RangedTag { name, parameters, content } => NorgAST::RangedTag { name, parameters, content } },
        select! { NorgASTFlat::InfirmTag { name, parameters } => NorgAST::InfirmTag { name, parameters } },
    ));

    let heading_content =
        move |current_level: u16,
              title: Vec<ParagraphSegmentToken>,
              extensions: Vec<DetachedModifierExtension>| {
            choice((
                select! {
                    NorgASTFlat::Heading { level, title, extensions } if level > current_level => (level, title, extensions),
                }.then(content.repeated())
                    .map(|((level, title, extensions), content)| {
                        NorgAST::Heading { level, title, extensions, content }
                    }),
                content,
            )).repeated().map(move |x| {
                    NorgAST::Heading {
                        level: current_level,
                        title: title.clone(),
                        extensions: extensions.clone(),
                        content: x,
                    }
                })
        };

    select! {
        NorgASTFlat::Heading { level, title, extensions } if level == current_level => (level, title, extensions)
    }.then(content.repeated()).map(|((lvl, title, extensions), content)| {
            NorgAST::Heading { level: lvl, title, extensions, content }
        }).repeated()
}

pub fn stage_4(
) -> impl Parser<NorgASTFlat, Vec<NorgAST>, Error = chumsky::error::Simple<NorgASTFlat>> {
    recursive(|stage_4| {
        let content = choice((
            select! { NorgASTFlat::Paragraph(segment) => NorgAST::Paragraph(segment) },
            // TODO: pull these out into their own thing
            select! {
                NorgASTFlat::NestableDetachedModifier { modifier_type, level, extensions, content } =>
                    NorgAST::NestableDetachedModifier { modifier_type, level, extensions, content }
            },
            select! {
                NorgASTFlat::RangeableDetachedModifier { modifier_type, title, extensions, content } =>
                    NorgAST::RangeableDetachedModifier { modifier_type, title, extensions, content }
            },
            select! {
                NorgASTFlat::CarryoverTag { tag_type, name, parameters, next_object } =>
                    NorgAST::CarryoverTag { tag_type, name, parameters, next_object }
            },
            select! { NorgASTFlat::RangedTag { name, parameters, content } => NorgAST::RangedTag { name, parameters, content } },
            select! { NorgASTFlat::InfirmTag { name, parameters } => NorgAST::InfirmTag { name, parameters } },
        ));

        // ...continuing from above: And then I kinda want to use this approach if it will work b/c
        // I think that it will be much faster than the other way, b/c it will fail earlier.
        let heading_content = move |current_level: u16, title: Vec<ParagraphSegmentToken>, extensions: Vec<DetachedModifierExtension>| {
            choice((
                select! {
                    NorgASTFlat::Heading { level, title, extensions } if level > current_level => (level, title, extensions),
                }.then(content.repeated())
                    .map(|((level, title, extensions), content)| {
                        NorgAST::Heading { level, title, extensions, content }
                    }),
                content,
            )).repeated().map(move |x| {
                    NorgAST::Heading {
                        level: current_level,
                        title: title.clone(),
                        extensions: extensions.clone(),
                        content: x,
                    }
                })
        };

        // this is the closest that I've gotten, it parses happy path, but stops parsing after it
        // sees some type of failure.
        select! {
            NorgASTFlat::Heading { level, title, extensions } => (level, title, extensions),
        }.then_with(move |(lvl, title, ext)| heading_content(lvl, title, ext))
            // .try_map(|ast, span| {
            //     if let NorgAST::Heading { level, content, .. } = ast.clone() {
            //         if content.iter().any(|item| {
            //             if let NorgAST::Heading { level: inner_level, .. } = item {
            //                 *inner_level <= level
            //             } else {
            //                 false
            //             }
            //         }) {
            //             return Err(Simple::custom(span, "Heading consumed an equal or lower level heading".to_string()));
            //         }
            //     }
            //     Ok(ast)
            // })

    })
        .repeated()
        .at_least(1)
}
