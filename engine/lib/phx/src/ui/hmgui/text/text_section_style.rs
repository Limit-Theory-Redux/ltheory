use std::ops::Range;

use parley::context::RangedBuilder;

use crate::render::Color;

use super::TextStyle;

#[derive(Clone, PartialEq)]
struct SectionStyle {
    range: Range<usize>,
    style: TextStyle,
}

impl SectionStyle {
    fn new(start: usize, end: usize, style: &TextStyle) -> Self {
        Self {
            range: Range { start, end },
            style: style.clone(),
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct TextSectionStyle {
    section_styles: Vec<SectionStyle>,
}

impl TextSectionStyle {
    pub fn is_empty(&self) -> bool {
        self.section_styles.is_empty()
    }

    pub fn add(&mut self, start: usize, end: usize, style: &TextStyle) {
        self.section_styles
            .push(SectionStyle::new(start, end, style));
    }

    pub fn apply<'a>(&'a self, builder: &mut RangedBuilder<'a, Color, &str>) {
        for section_style in &self.section_styles {
            section_style
                .style
                .apply_to_section(builder, &section_style.range);
        }
    }

    pub fn update(&mut self, pos: usize, removed: usize, added: usize) {
        if self.section_styles.is_empty() {
            return;
        }

        if removed == 0 {
            debug_assert!(added > 0, "No changes made");

            // text was inserted -> expand corresponding style sections
            for section_style in &mut self.section_styles {
                if section_style.range.start >= pos {
                    section_style.range.start += added;
                }

                if section_style.range.end > pos {
                    section_style.range.end += added;
                }
            }
        } else {
            let (offset, inc) = if added == 0 {
                // selected text was removed -> remove and update corresponding sections
                (removed, false)
            } else {
                // selected text was replaced with another one -> remove and update influenced sections
                if removed > added {
                    (removed - added, false)
                } else {
                    (added - removed, true)
                }
            };

            let removed_start = pos;
            let removed_end = pos + removed;
            let mut section_styles = vec![];

            while let Some(mut section_style) = self.section_styles.pop() {
                if section_style.range.end <= removed_start {
                    // section is before removed selection -> keep it
                    //               [------------]  removed selection
                    //  [--------]                   section
                    //  [--------]                   result

                    section_styles.push(section_style);
                } else if section_style.range.start < removed_start {
                    if section_style.range.end <= removed_end {
                        //        [------------]  removed selection
                        //  [-----.--]            section
                        //  [-----]               result

                        section_style.range.end = removed_start;
                        section_styles.push(section_style);
                    } else {
                        //       [------------]     removed selection
                        //  [----.------------.--]  section
                        //  [------]                result

                        if inc {
                            section_style.range.end += offset;
                        } else {
                            section_style.range.end -= offset;
                        }

                        section_styles.push(section_style);
                    }
                } else if section_style.range.start <= removed_end {
                    if section_style.range.end > removed_end {
                        //  [------------]     removed selection
                        //         [-----.--]  section
                        //  [--]               result

                        section_style.range.start = removed_start;

                        if inc {
                            section_style.range.end += offset;
                        } else {
                            section_style.range.end -= offset;
                        }

                        section_styles.push(section_style);
                    }

                    //  [------------]  removed selection
                    //     [-----]      section
                    //                  result
                } else {
                    //  [------------]              removed selection
                    //                  [--------]  section
                    //  [--------]                  result

                    if inc {
                        section_style.range.start += offset;
                        section_style.range.end += offset;
                    } else {
                        section_style.range.start -= offset;
                        section_style.range.end -= offset;
                    }

                    section_styles.push(section_style);
                }
            }

            self.section_styles = section_styles;
        }
    }
}
