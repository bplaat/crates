/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#define QOI_IMPLEMENTATION
#import "qoi.h"

#import <Cocoa/Cocoa.h>
#import <QuickLookUI/QuickLookUI.h>

/* NSExtensionMain is provided by the extension runtime but has no public header. */
extern int NSExtensionMain(int argc, const char** argv);

// MARK: Utils
static NSImage* qoi_to_nsimage(uint8_t* pixels, QoiDesc desc) {
    NSInteger w = (NSInteger)desc.width;
    NSInteger h = (NSInteger)desc.height;

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

    NSBitmapImageRep* srgbRep =
        [rep bitmapImageRepByConvertingToColorSpace:[NSColorSpace sRGBColorSpace]
                                   renderingIntent:NSColorRenderingIntentDefault];

    NSImage* image = [[NSImage alloc] initWithSize:NSMakeSize((CGFloat)w, (CGFloat)h)];
    [image addRepresentation:srgbRep ? srgbRep : rep];
    return image;
}

// MARK: PreviewViewController
@interface PreviewViewController : NSViewController <QLPreviewingController>
@property(strong, nonatomic) NSImageView* imageView;
@end

@implementation PreviewViewController

- (void)loadView {
    NSImageView* imageView = [[NSImageView alloc] initWithFrame:NSMakeRect(0, 0, 600, 400)];
    imageView.imageScaling = NSImageScaleProportionallyDown;
    imageView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    self.imageView = imageView;
    self.view = imageView;
}

- (void)preparePreviewOfFileAtURL:(NSURL*)url
                completionHandler:(void (^)(NSError* _Nullable))completionHandler {
    NSError* readError = nil;
    NSData* data = [NSData dataWithContentsOfURL:url options:0 error:&readError];
    if (!data) {
        dispatch_async(dispatch_get_main_queue(), ^{ completionHandler(readError); });
        return;
    }

    QoiDesc desc;
    uint8_t* pixels = qoi_decode(data.bytes, (int)data.length, &desc);
    if (!pixels) {
        NSError* err = [NSError
            errorWithDomain:NSCocoaErrorDomain
                       code:NSFileReadCorruptFileError
                   userInfo:@{NSLocalizedDescriptionKey : @"The file is not a valid QOI image."}];
        dispatch_async(dispatch_get_main_queue(), ^{ completionHandler(err); });
        return;
    }

    NSImage* image = qoi_to_nsimage(pixels, desc);
    CGFloat w = (CGFloat)desc.width;
    CGFloat h = (CGFloat)desc.height;

    dispatch_async(dispatch_get_main_queue(), ^{
        [self loadViewIfNeeded];
        self.imageView.image = image;
        /* Tell Quick Look the natural pixel size so the panel fits the image. */
        self.preferredContentSize = NSMakeSize(w, h);
        completionHandler(nil);
    });
}

@end

// MARK: Main
int main(int argc, const char** argv) {
    return NSExtensionMain(argc, (const char**)argv);
}
