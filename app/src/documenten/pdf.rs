//! Pdf: CoreGraphics tekent elke pagina als afbeelding, alleen de pagina's in beeld, op de
//! achtergrond. Een handvol blijft bewaard; wat verder weg is, gaat uit het geheugen.

use std::{
    collections::{HashMap, HashSet},
    ffi::c_void,
    path::{Path, PathBuf},
    ptr::NonNull,
    sync::Arc,
};

use gpui_kit::*;
use objc2_core_foundation::{CFRetained, CFURL, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGColorSpace, CGContext, CGPDFBox, CGPDFDocument, CGPDFPage};

use crate::taal::t;
use crate::thema::{self as t};

/// Zo breed staat een pagina hoogstens in beeld.
const MAX_BREED: f32 = 900.0;
/// Zoveel pixels breed tekent CoreGraphics een pagina: scherp op een Retina-scherm.
const PIXELS: usize = 1800;
/// Zoveel getekende pagina's blijven bewaard.
const BEWAAR: usize = 12;
/// Ruimte rond een pagina.
const MARGE: f32 = 24.0;

/// De maat van elke pagina in punten, breedte en hoogte, zoals hij in beeld staat.
pub fn lees(pad: &Path) -> Result<Vec<(f32, f32)>, String> {
    let fout = || t("Kan deze pdf niet lezen.", "Cannot read this pdf.").to_string();
    let doc = document(pad).ok_or_else(fout)?;
    let aantal = CGPDFDocument::number_of_pages(Some(&doc));
    let maten: Vec<(f32, f32)> =
        (1..=aantal).filter_map(|n| CGPDFDocument::page(Some(&doc), n)).map(|pagina| maat(&pagina)).collect();
    if maten.is_empty() {
        return Err(fout());
    }
    Ok(maten)
}

fn document(pad: &Path) -> Option<CFRetained<CGPDFDocument>> {
    let url = CFURL::from_file_path(pad)?;
    CGPDFDocument::with_url(Some(&url))
}

fn maat(pagina: &CGPDFPage) -> (f32, f32) {
    let vak = CGPDFPage::box_rect(Some(pagina), CGPDFBox::CropBox);
    let (b, h) = (vak.size.width as f32, vak.size.height as f32);
    match CGPDFPage::rotation_angle(Some(pagina)).rem_euclid(360) {
        90 | 270 => (h, b),
        _ => (b, h),
    }
}

unsafe extern "C-unwind" {
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        space: Option<&CGColorSpace>,
        bitmap_info: u32,
    ) -> Option<NonNull<CGContext>>;
}

/// kCGImageAlphaPremultipliedFirst met kCGBitmapByteOrder32Little: BGRA, zoals GPUI het wil.
const BGRA: u32 = 2 | (2 << 12);

/// Tekent pagina `index` (vanaf 0) op een wit vlak van `PIXELS` breed.
fn teken(pad: &Path, index: usize) -> Option<image::RgbaImage> {
    let doc = document(pad)?;
    let pagina = CGPDFDocument::page(Some(&doc), index + 1)?;
    let (b, h) = maat(&pagina);
    let schaal = PIXELS as f64 / b as f64;
    let hoog = (h as f64 * schaal).round().max(1.0) as usize;
    let mut buffer = vec![0u8; PIXELS * hoog * 4];
    let ruimte = CGColorSpace::new_device_rgb()?;
    // Veilig: de buffer is precies zo groot als breedte × hoogte × 4 en leeft langer dan de context.
    let context = unsafe {
        let ptr = CGBitmapContextCreate(buffer.as_mut_ptr().cast(), PIXELS, hoog, 8, PIXELS * 4, Some(&ruimte), BGRA)?;
        CFRetained::from_raw(ptr)
    };
    let c = Some(&*context);
    CGContext::set_rgb_fill_color(c, 1.0, 1.0, 1.0, 1.0);
    CGContext::fill_rect(c, CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(PIXELS as f64, hoog as f64)));
    CGContext::scale_ctm(c, schaal, schaal);
    let vak = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(b as f64, h as f64));
    CGContext::concat_ctm(c, CGPDFPage::drawing_transform(Some(&pagina), CGPDFBox::CropBox, vak, 0, true));
    CGContext::draw_pdf_page(c, Some(&pagina));
    drop(context);
    image::RgbaImage::from_raw(PIXELS as u32, hoog as u32, buffer)
}

pub struct Pdf {
    pad: PathBuf,
    maten: Vec<(f32, f32)>,
    lijst: ListState,
    /// De breedte van een pagina in beeld, bij het vorige tekenen.
    breed: f32,
    paginas: HashMap<usize, Arc<RenderImage>>,
    bezig: HashSet<usize>,
    /// Afbeeldingen die uit het geheugen van de GPU moeten, bij het volgende tekenen.
    weg: Vec<Arc<RenderImage>>,
    /// Telt op bij elke nieuwe versie van het bestand, zodat een oude tekening niet terugkomt.
    versie: usize,
}

impl Pdf {
    pub fn new(pad: PathBuf, maten: Vec<(f32, f32)>) -> Self {
        let lijst = ListState::new(maten.len(), ListAlignment::Top, px(1000.0));
        Self { pad, maten, lijst, breed: 0.0, paginas: HashMap::new(), bezig: HashSet::new(), weg: Vec::new(), versie: 0 }
    }

    /// Het bestand veranderde: alle pagina's opnieuw, de plek blijft.
    pub fn vervang(&mut self, maten: Vec<(f32, f32)>, cx: &mut Context<Self>) {
        let oud = self.maten.len();
        self.maten = maten;
        self.versie += 1;
        self.weg.extend(self.paginas.drain().map(|(_, beeld)| beeld));
        self.bezig.clear();
        self.lijst.splice(0..oud, self.maten.len());
        cx.notify();
    }

    /// De tab gaat dicht: alles uit het geheugen van de GPU.
    pub fn ruim_op(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.weg.extend(self.paginas.drain().map(|(_, beeld)| beeld));
        for beeld in self.weg.drain(..) {
            cx.drop_image(beeld, Some(window));
        }
    }

    fn vraag(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.bezig.insert(index) {
            return;
        }
        let pad = self.pad.clone();
        let versie = self.versie;
        cx.spawn(async move |this, cx| {
            let beeld = cx.background_executor().spawn(async move { teken(&pad, index) }).await;
            let _ = this.update(cx, |pdf, cx| {
                if pdf.versie != versie {
                    return;
                }
                if let Some(beeld) = beeld {
                    let beeld = Arc::new(RenderImage::new(vec![image::Frame::new(beeld)]));
                    pdf.paginas.insert(index, beeld);
                    pdf.vergeet(index);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Houdt de `BEWAAR` pagina's die het dichtst bij `index` liggen.
    fn vergeet(&mut self, index: usize) {
        while self.paginas.len() > BEWAAR {
            let Some(verst) = self.paginas.keys().copied().max_by_key(|p| p.abs_diff(index)) else { break };
            if let Some(beeld) = self.paginas.remove(&verst) {
                self.weg.push(beeld);
            }
            self.bezig.remove(&verst);
        }
    }

    fn pagina(&mut self, index: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let (b, h) = self.maten.get(index).copied().unwrap_or((1.0, 1.0));
        let hoog = self.breed * h / b;
        let beeld = self.paginas.get(&index).cloned();
        if beeld.is_none() {
            self.vraag(index, cx);
        }
        div()
            .w_full()
            .flex()
            .justify_center()
            .pt(px(if index == 0 { MARGE } else { 0.0 }))
            .pb(px(MARGE))
            .child(
                div()
                    .w(px(self.breed))
                    .h(px(hoog))
                    .bg(rgb(0xffffff))
                    .children(beeld.map(|beeld| img(beeld).size_full().object_fit(ObjectFit::Fill))),
            )
            .into_any_element()
    }
}

impl Render for Pdf {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        for beeld in self.weg.drain(..) {
            cx.drop_image(beeld, Some(window));
        }
        let breed = (f32::from(window.viewport_size().width) - 2.0 * MARGE).clamp(100.0, MAX_BREED);
        if breed != self.breed {
            self.breed = breed;
            self.lijst.remeasure();
        }
        let th = t::thema(window);
        div()
            .size_full()
            .bg(rgb(th.bg))
            .child(list(self.lijst.clone(), cx.processor(Self::pagina)).size_full())
    }
}
