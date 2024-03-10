using Microsoft.UI.Composition;
using Microsoft.UI.Xaml;

namespace WinUIEx
{
    /// <summary>
    /// Helper class for creating composition-brush based backdrops.
    /// </summary>
    public abstract class CompositionBrushBackdrop : Microsoft.UI.Xaml.Media.SystemBackdrop
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="CompositionBrushBackdrop"/> class.
        /// </summary>
        public CompositionBrushBackdrop() 
        {
        }

        /// <summary>
        /// Called when the brush needs to be created for the provided compositor.
        /// </summary>
        /// <param name="compositor">Compositor context</param>
        /// <returns>Brush</returns>
        protected abstract Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor);
        
        /// <inheritdoc />
        protected override void OnDefaultSystemBackdropConfigurationChanged(ICompositionSupportsSystemBackdrop target, XamlRoot xamlRoot)
        {
            if (target != null)
                base.OnDefaultSystemBackdropConfigurationChanged(target, xamlRoot);
        }

        /// <inheritdoc />
        protected override void OnTargetConnected(ICompositionSupportsSystemBackdrop connectedTarget, XamlRoot xamlRoot)
        {
            connectedTarget.SystemBackdrop = CreateBrush(WindowManager.Compositor);
            base.OnTargetConnected(connectedTarget, xamlRoot);
        }

        /// <inheritdoc />
        protected override void OnTargetDisconnected(ICompositionSupportsSystemBackdrop disconnectedTarget)
        {
            var backdrop = disconnectedTarget.SystemBackdrop;
            disconnectedTarget.SystemBackdrop = null;
            backdrop?.Dispose();
            base.OnTargetDisconnected(disconnectedTarget);
        }
    }
}
