use serde::Serialize;

use chumsky::prelude::*;
use chumsky::Parser;

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

        // take a single heading followed by everything that isn't a heading, and throw that into
        // the heading's content in a Non-flat AST

        // Okay, so I have this thing, and it should probably hopefully work to parse headings and
        // nest them properly. But this doesn't work with lists right now
        // (NestableDetachedModifiers). I think that I will just have to mimic this logic with
        // lists.

        // NOTE: this doesn't really work, I'm not sure why it complains about the types
        // let heading_with_paragraphs = select! {
        //     NorgASTFlat::Heading { level, title, extensions } => (level, title, extensions),
        // }.then(content.repeated().then(stage_4.clone()))
        //     // temp thing
        //     // .map(|((level, title, extensions), (c1, c2))| {
        //     //     let content = [c1, c2].concat();
        //     //     NorgAST::Heading { level, title, extensions, content }
        //     // });
        //
        //     .try_map(|((level, title, extensions), (c1, c2)), span| {
        //         let content = [c1, c2].concat();
        //         if content
        //             .iter()
        //             .any(|x| match x {
        //                 NorgAST::Heading { level: inner_level, title, extensions, content } if level <= *inner_level => true,
        //                 _ => false
        //             }) {
        //             Err(Simple::custom(span, "We should never fail to parse entirely due to this error".to_string()))
        //         } else {
        //             Ok(NorgAST::Heading { level, title, extensions, content })
        //         }
        //     });

        // ...continuing from above: And then I kinda want to use this approach if it will work b/c
        // I think that it will be much faster than the other way, b/c it will fail earlier.
        // let n_level_heading = |current_level: u16| {
        //     select! {
        //         NorgASTFlat::Heading { level, title, extensions } if level > current_level => (level, title, extensions),
        //     }.repeated()
        // };


        // NOTE: working
        // select! {
        //     NorgASTFlat::Heading { level, title, extensions } => (level, title, extensions),
        // }.then(content.repeated())
        //     .map(|((lv, t, ext), content)| {
        //         NorgAST::Heading { level: lv, title: t, extensions: ext, content }
        //     })

        // NOTE: working
        select! {
            NorgASTFlat::Heading { level, title, extensions } => (level, title, extensions),
        }.then(content.repeated())
            .try_map(|((lv, t, ext), content), _span| {
                Ok(NorgAST::Heading { level: lv, title: t, extensions: ext, content })
            })

        // this is working too (type wise, not in practice)
        // select! {
        //     NorgASTFlat::Heading { level, title, extensions } => (level, title, extensions),
        // }.then(content.repeated()).then(stage_4)
        //     .try_map(|(((lv, t, ext), c1), c2), _span| {
        //         let content = [c1, vec![c2]].concat();
        //         Ok(NorgAST::Heading { level: lv, title: t, extensions: ext, content })
        //     })

        // select! {
        //     NorgASTFlat::Heading { level, title, extensions } => (level, title, extensions),
        // }.then(content.repeated()).then(stage_4)
        //     .try_map(|(((lv, t, ext), c1), c2), _span| {
        //         let content = [c1, vec![c2]].concat();
        //         Ok(NorgAST::Heading { level: lv, title: t, extensions: ext, content })
        //     })

    })
        .repeated()
        .at_least(1)
}
