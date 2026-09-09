// Mechanical platform exports of the approved AI artwork. Run from repository root.
import Foundation
import CoreGraphics
import ImageIO

let root = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
func load(_ path: String) -> CGImage {
    let source = CGImageSourceCreateWithURL(root.appendingPathComponent(path) as CFURL, nil)!
    return CGImageSourceCreateImageAtIndex(source, 0, nil)!
}
func export(_ source: CGImage, _ size: Int, _ path: String, mac: Bool = false, tray: Bool = false) {
    let ctx = CGContext(data: nil, width: size, height: size, bitsPerComponent: 8, bytesPerRow: size * 4,
                        space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
    ctx.interpolationQuality = .high
    let side = CGFloat(size)
    var rect = CGRect(x: 0, y: 0, width: side, height: side)
    if mac {
        rect = rect.insetBy(dx: side * 0.08, dy: side * 0.08)
        ctx.addPath(CGPath(roundedRect: rect, cornerWidth: side * 0.185, cornerHeight: side * 0.185, transform: nil))
        ctx.clip()
    }
    // Enlarge the transparent foreground for a legible 20 pt menu bar symbol.
    if tray { rect = rect.insetBy(dx: -side * 0.27, dy: -side * 0.27) }
    ctx.draw(source, in: rect)
    let url = root.appendingPathComponent(path)
    try! FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    let dst = CGImageDestinationCreateWithURL(url as CFURL, "public.png" as CFString, 1, nil)!
    CGImageDestinationAddImage(dst, ctx.makeImage()!, nil)
    precondition(CGImageDestinationFinalize(dst))
    if tray {
        // Tauri template images use only alpha; raw RGBA avoids an extra PNG decoder feature.
        try! Data(bytes: ctx.data!, count: size * size * 4).write(to: url.deletingPathExtension().appendingPathExtension("rgba"))
    }
}
let full = load("design/android-icon/voice-cursor-master.png")
let foreground = load("design/android-icon/voice-cursor-foreground.png")
export(foreground, 432, "android/app/src/main/res/drawable-nodpi/ic_launcher_foreground.png")
export(full, 192, "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png")
export(full, 64, "src-tauri/icons/64x64.png")
export(foreground, 40, "src-tauri/icons/tray-template.png", tray: true)
for size in [16, 32, 128, 256, 512] {
    export(full, size, "design/android-icon/macos.iconset/icon_\(size)x\(size).png", mac: true)
    export(full, size * 2, "design/android-icon/macos.iconset/icon_\(size)x\(size)@2x.png", mac: true)
}
