/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#define QOI_IMPLEMENTATION
#import "qoi.h"

#import <Cocoa/Cocoa.h>

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

// MARK: QoiWindowController
@interface QoiWindowController : NSWindowController
@property(strong, nonatomic) NSImageView* imageView;
@end

@implementation QoiWindowController

- (instancetype)init {
    NSWindow* window = [[NSWindow alloc]
        initWithContentRect:NSMakeRect(0, 0, 600, 400)
                  styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable |
                            NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskResizable
                    backing:NSBackingStoreBuffered
                      defer:NO];
    self = [super initWithWindow:window];
    if (self) {
        self.imageView = [[NSImageView alloc] initWithFrame:window.contentView.bounds];
        self.imageView.imageScaling = NSImageScaleProportionallyUpOrDown;
        self.imageView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
        [window.contentView addSubview:self.imageView];
    }
    return self;
}

- (void)setImage:(NSImage*)image title:(NSString*)title {
    self.imageView.image = image;
    self.window.title = title;

    NSSize size = image.size;
    NSRect frame = self.window.frame;
    frame.size.width = MAX(320, MIN(size.width, 1280));
    frame.size.height = MAX(240, MIN(size.height, 960));
    [self.window setFrame:frame display:YES animate:NO];
    [self.window center];
}

@end

// MARK: QoiDocument
@interface QoiDocument : NSDocument
@property(strong, nonatomic) NSImage* image;
@end

@implementation QoiDocument

- (BOOL)readFromData:(NSData*)data ofType:(NSString*)typeName error:(NSError**)outError {
    (void)typeName;
    QoiDesc desc;
    uint8_t* pixels = qoi_decode(data.bytes, (int)data.length, &desc);
    if (!pixels) {
        if (outError) {
            *outError = [NSError errorWithDomain:NSCocoaErrorDomain
                                            code:NSFileReadCorruptFileError
                                        userInfo:@{
                                            NSLocalizedDescriptionKey :
                                                @"The file is not a valid QOI image."
                                        }];
        }
        return NO;
    }

    /* pixels is freed by kCFAllocatorMalloc inside qoi_to_nsimage. */
    self.image = qoi_to_nsimage(pixels, desc);
    return YES;
}

- (void)makeWindowControllers {
    QoiWindowController* wc = [QoiWindowController new];
    [self addWindowController:wc];
    NSString* title = self.fileURL ? self.fileURL.lastPathComponent : @"QOI Image";
    [wc setImage:self.image title:title];
    [wc showWindow:self];
}

+ (BOOL)autosavesInPlace {
    return NO;
}

@end

// MARK: AppDelegate
@interface AppDelegate : NSObject <NSApplicationDelegate>
@end

@implementation AppDelegate

- (void)applicationWillFinishLaunching:(NSNotification*)notification {
    (void)notification;

    // Build menu bar
    NSMenu* menuBar = [NSMenu new];
    NSApplication.sharedApplication.mainMenu = menuBar;

    // App menu
    NSMenuItem* appItem = [NSMenuItem new];
    [menuBar addItem:appItem];
    NSMenu* appMenu = [NSMenu new];
    appItem.submenu = appMenu;
    [appMenu addItemWithTitle:@"Quit QoiViewer" action:@selector(terminate:) keyEquivalent:@"q"];

    // File menu
    NSMenuItem* fileItem = [NSMenuItem new];
    [menuBar addItem:fileItem];
    NSMenu* fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
    fileItem.submenu = fileMenu;
    [fileMenu addItemWithTitle:@"Open..." action:@selector(openDocument:) keyEquivalent:@"o"];
    [fileMenu addItemWithTitle:@"Close" action:@selector(performClose:) keyEquivalent:@"w"];
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication*)sender {
    (void)sender;
    return YES;
}

- (BOOL)applicationOpenUntitledFile:(NSApplication*)sender {
    (void)sender;
    // Show open panel when launched without a file argument
    [[NSDocumentController sharedDocumentController] openDocument:nil];
    return YES;
}

@end

// MARK: Main
int main(int argc, const char** argv) {
    NSApplication* app = [NSApplication sharedApplication];
    app.delegate = [AppDelegate new];
    return NSApplicationMain(argc, argv);
}

