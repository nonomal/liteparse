use crate::error::PdfiumError;
use crate::ffi;
use crate::library::Library;
use crate::page::Page;

/// An open PDF document.
///
/// The `'lib` lifetime ties this `Document` to the [`Library`] that opened
/// it, statically guaranteeing that no PDFium calls happen after the
/// process-wide PDFium lock has been released.
pub struct Document<'lib> {
    pub(crate) handle: pdfium_sys::FPDF_DOCUMENT,
    pub(crate) _lib: std::marker::PhantomData<&'lib Library>,
}

impl<'lib> Document<'lib> {
    pub fn page_count(&self) -> i32 {
        unsafe { ffi!(FPDF_GetPageCount(self.handle)) }
    }

    pub fn page(&self, index: i32) -> Result<Page<'_, 'lib>, PdfiumError> {
        let handle = unsafe { ffi!(FPDF_LoadPage(self.handle, index)) };
        if handle.is_null() {
            return Err(PdfiumError::PageNotFound);
        }
        Ok(Page {
            handle,
            doc_handle: self.handle,
            _doc: std::marker::PhantomData,
        })
    }
}

impl Drop for Document<'_> {
    fn drop(&mut self) {
        unsafe { ffi!(FPDF_CloseDocument(self.handle)) };
    }
}
