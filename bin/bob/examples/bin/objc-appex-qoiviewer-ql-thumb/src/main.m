/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#define QOI_IMPLEMENTATION
#import "qoi.h"

#import <Cocoa/Cocoa.h>
#import <QuickLookThumbnailing/QuickLookThumbnailing.h>

/* NSExtensionMain is provided by the extension runtime but has no public header. */
extern int NSExtensionMain(int argc, const char** argv);

// MARK: Utils
static NSImage* qoi_to_nsimage(uint8_t* pixels, QoiDesc desc) {
    NSInteger w = (NSInteger)desc.width;
    NSInteger h = (NSInteger)desc.height;

    /* NSBitmapFormatAlphaNonpremultiplied: QOI stores un-premultiplied alpha. */
    NSBitmapImageRep* rep = [[NSBitmapImageRep alloc]
        initWithBitmapDataPlanes:NULL
                      pixelsWide:w
                      pixelsHigh:h
                   bitsPerSample:8
                 samplesPerPixel:4
                        hasAlpha:YES
                        isPlanar:NO
                  colorSpaceName:NSDeviceRGBColorSpace
                    bitmapFormat:NSBitmapFormatAlphaNonpremultiplied
                     bytesPerRow:w * 4
                    bitsPerPixel:32];
    memcpy(rep.bitmapData, pixels, (size_t)(w * h * 4));
    free(pixels);

    /* Re-tag the rep with the sRGB color space (QOI default: colorspace==0 = sRGB). */
    NSBitmapImageRep* srgbRep =
        [rep bitmapImageRepByConvertingToColorSpace:[NSColorSpace sRGBColorSpace]
                                   renderingIntent:NSColorRenderingIntentDefault];

    NSImage* image = [[NSImage alloc] initWithSize:NSMakeSize((CGFloat)w, (CGFloat)h)];
    [image addRepresentation:srgbRep ? srgbRep : rep];
    return image;
}

// MARK: ThumbnailProvider
@interface ThumbnailProvider : QLThumbnailProvider
@end

@implementation ThumbnailProvider

- (void)provideThumbnailForFileRequest:(QLFileThumbnailRequest*)request
                     completionHandler:
                         (void (^)(QLThumbnailReply* _Nullable, NSError* _Nullable))handler {
    NSError* readErr = nil;
    NSData* data = [NSData dataWithContentsOfURL:request.fileURL options:0 error:&readErr];
    if (!data) {
        handler(nil, readErr);
        return;
    }

    QoiDesc desc;
    uint8_t* pixels = qoi_decode(data.bytes, (int)data.length, &desc);
    if (!pixels) {
        handler(nil, [NSError errorWithDomain:NSCocoaErrorDomain
                                        code:NSFileReadCorruptFileError
                                    userInfo:@{
                                        NSLocalizedDescriptionKey : @"The file is not a valid QOI image."
                                    }]);
        return;
    }

    NSImage* image = qoi_to_nsimage(pixels, desc);

    /* Compute context size (in points) proportionally fitted to maximumSize. */
    CGFloat imgW = (CGFloat)desc.width / request.scale;
    CGFloat imgH = (CGFloat)desc.height / request.scale;
    CGFloat maxW = request.maximumSize.width;
    CGFloat maxH = request.maximumSize.height;
    CGFloat ratio = MIN(maxW / imgW, MIN(maxH / imgH, 1.0));
    CGSize contextSize = CGSizeMake(ceil(imgW * ratio), ceil(imgH * ratio));

    QLThumbnailReply* reply = [QLThumbnailReply
        replyWithContextSize:contextSize
        currentContextDrawingBlock:^BOOL {
            [image drawInRect:NSMakeRect(0, 0, contextSize.width, contextSize.height)];
            return YES;
        }];
    handler(reply, nil);
}

@end

// MARK: Main
int main(int argc, const char** argv) {
    return NSExtensionMain(argc, (const char**)argv);
}
