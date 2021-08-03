#[doc = "Reader of register CCB%s_DITH5"]
pub type R = crate::R<u32, super::CCB_DITH5>;
#[doc = "Writer for register CCB%s_DITH5"]
pub type W = crate::W<u32, super::CCB_DITH5>;
#[doc = "Register CCB%s_DITH5 `reset()`'s with value 0"]
impl crate::ResetValue for super::CCB_DITH5 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `DITHERCYB`"]
pub type DITHERCYB_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DITHERCYB`"]
pub struct DITHERCYB_W<'a> {
    w: &'a mut W,
}
impl<'a> DITHERCYB_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x1f) | ((value as u32) & 0x1f);
        self.w
    }
}
#[doc = "Reader of field `CCB`"]
pub type CCB_R = crate::R<u32, u32>;
#[doc = "Write proxy for field `CCB`"]
pub struct CCB_W<'a> {
    w: &'a mut W,
}
impl<'a> CCB_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0007_ffff << 5)) | (((value as u32) & 0x0007_ffff) << 5);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:4 - Dithering Buffer Cycle Number"]
    #[inline(always)]
    pub fn dithercyb(&self) -> DITHERCYB_R {
        DITHERCYB_R::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 5:23 - Channel Compare/Capture Buffer Value"]
    #[inline(always)]
    pub fn ccb(&self) -> CCB_R {
        CCB_R::new(((self.bits >> 5) & 0x0007_ffff) as u32)
    }
}
impl W {
    #[doc = "Bits 0:4 - Dithering Buffer Cycle Number"]
    #[inline(always)]
    pub fn dithercyb(&mut self) -> DITHERCYB_W {
        DITHERCYB_W { w: self }
    }
    #[doc = "Bits 5:23 - Channel Compare/Capture Buffer Value"]
    #[inline(always)]
    pub fn ccb(&mut self) -> CCB_W {
        CCB_W { w: self }
    }
}
