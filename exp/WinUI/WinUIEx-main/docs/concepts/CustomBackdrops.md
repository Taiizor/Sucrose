## Custom Backdrops

The [TransparentTintBackdrop](https://dotmorten.github.io/WinUIEx/api/WinUIEx.TransparentTintBackdrop.html) and its [CompositionBrushBackdrop](https://dotmorten.github.io/WinUIEx/api/WinUIEx.CompositionBrushBackdrop.html) baseclass allows you to set the background to be fully transparent, or any composition brush.
This can be useful for changing the visual shape of the window, and create transparent areas of the window, or more advanced background effects.
These are similar to the Mica and Acrylic backdrops.

### Transparent backdrop:
```xml
<Window ...
   xmlns:winuiex="using:WinUIEx">
    <Window.SystemBackdrop>
        <winuiex:TransparentTintBackdrop />
    </Window.SystemBackdrop>
</Window>
```

### Semi-transparent blue backdrop:
```xml
<Window ...
   xmlns:winuiex="using:WinUIEx">
    <Window.SystemBackdrop>
        <winuiex:TransparentTintBackdrop TintColor="#554444ff" />
    </Window.SystemBackdrop>
</Window>
```

### Custom animated composition-brush backdrop:
```cs
public class ColorAnimatedBackdrop : CompositionBrushBackdrop
{
    protected override Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor)
    {
        var brush = compositor.CreateColorBrush(Windows.UI.Color.FromArgb(255,255,0,0));
        var animation = compositor.CreateColorKeyFrameAnimation();
        var easing = compositor.CreateLinearEasingFunction();
        animation.InsertKeyFrame(0, Colors.Red, easing);
        animation.InsertKeyFrame(.333f, Colors.Green, easing);
        animation.InsertKeyFrame(.667f, Colors.Blue, easing);
        animation.InsertKeyFrame(1, Colors.Red, easing);
        animation.InterpolationColorSpace = Windows.UI.Composition.CompositionColorSpace.Hsl;
        animation.Duration = TimeSpan.FromSeconds(15);
        animation.IterationBehavior = Windows.UI.Composition.AnimationIterationBehavior.Forever;
        brush.StartAnimation("Color", animation);
        return brush;
    }
}
```

### Blurred composition-brush backdrop:
```cs
public class BlurredBackdrop : CompositionBrushBackdrop
{
    protected override Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor)
        => compositor.CreateHostBackdropBrush();
}
```

> [!Video https://github.com/dotMorten/WinUIEx/assets/1378165/49274b59-5628-4a0c-b6ca-4f38784614cf]
