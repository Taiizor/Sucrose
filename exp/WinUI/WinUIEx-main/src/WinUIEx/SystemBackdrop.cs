using Microsoft.UI.Composition;
using Microsoft.UI.Composition.SystemBackdrops;
using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using Windows.Win32.Graphics.Direct3D11;

namespace WinUIEx
{
    /// <summary>
    /// The backdrop configuration for applying a backdrop.
    /// </summary>
    /// <seealso cref="WindowEx.Backdrop"/>
    /// <seealso cref="MicaSystemBackdrop"/>
    /// <seealso cref="AcrylicSystemBackdrop"/>
    /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController" />
    /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController" />
    [Obsolete("Use Microsoft.UI.Xaml.Media.SystemBackdrop")]
    [EditorBrowsable(EditorBrowsableState.Never)]
    public abstract class SystemBackdrop
    {
        /// <summary>
        /// Resets any customized properties to their system defaults and reverts to automatic light/dark theme handling.
        /// </summary>
        public virtual void ResetProperties()
        {
            _isDarkTintOverridden = false;
            _isLightTintOverridden = false;
            _isDarkFallbackOverridden = false;
            _isLightFallbackOverridden = false;
            _isDarkTintOpacityOverridden = false;
            _isLightTintOpacityOverridden = false;
            _isDarkLuminosityOpacityOverridden = false;
            _isLightLuminosityOpacityOverridden = false;
            _darkTintColor = Windows.UI.Color.FromArgb(0, 0, 0, 0);
            _lightTintColor = Windows.UI.Color.FromArgb(0, 0, 0, 0);
            _darkFallbackColor = _defaultDarkFallback;
            _lightFallbackColor = _defaultLightFallback;
            _darkTintOpacity = 0;
            _lightTintOpacity = 0;
            _lightLuminosityOpacity = 1;
            _darkLuminosityOpacity = 1;
        }

        private protected Windows.UI.Color _darkTintColor = Windows.UI.Color.FromArgb(0, 0, 0, 0);
        private protected bool _isDarkTintOverridden = false;

        /// <summary>
        /// Gets or sets the color tint for the backdrop material.
        /// </summary>
        /// <value>The color tint for the background material.</value>
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController.TintColor" />
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.TintColor" />
        public Windows.UI.Color DarkTintColor
        {
            get => _darkTintColor;
            set
            {
                _isDarkTintOverridden = true;
                if (_darkTintColor != value)
                {
                    _darkTintColor = value;
                    NotifyDirty();
                }
            }
        }

        private protected Windows.UI.Color _lightTintColor = Windows.UI.Color.FromArgb(0, 0, 0, 0);
        private protected bool _isLightTintOverridden = false;

        /// <summary>
        /// Gets or sets the color tint for the backdrop material.
        /// </summary>
        /// <value>The color tint for the background material.</value>
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController.TintColor" />
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.TintColor" />
        public Windows.UI.Color LightTintColor
        {
            get => _lightTintColor;
            set
            {
                _isLightTintOverridden = true;
                if (_lightTintColor != value)
                {
                    _lightTintColor = value;
                    NotifyDirty();
                }
            }
        }

        private readonly protected static Windows.UI.Color _defaultDarkFallback = Windows.UI.Color.FromArgb(0xff, 0x20, 0x20, 0x20);
        private protected Windows.UI.Color _darkFallbackColor = _defaultDarkFallback;
        private protected bool _isDarkFallbackOverridden = false;

        /// <summary>
        /// Gets or sets the solid color to use when system conditions prevent rendering the backdrop material.
        /// </summary>
        /// <remarks>
        /// The backdrop material is replaced with a solid color when one of the fallback conditions is met, such as entering battery saver mode.
        /// </remarks>
        /// <value>The solid color to use when system conditions prevent rendering the backdrop material.</value>
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController.FallbackColor" />
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.FallbackColor" />
        public Windows.UI.Color DarkFallbackColor
        {
            get => _darkFallbackColor;
            set
            {
                _isDarkFallbackOverridden = true;
                if (_darkFallbackColor != value)
                {
                    _darkFallbackColor = value;
                    NotifyDirty();
                }
            }
        }
        
        private readonly protected static Windows.UI.Color _defaultLightFallback = Windows.UI.Color.FromArgb(0xff, 0xf3, 0xf3, 0xf3);
        private protected Windows.UI.Color _lightFallbackColor = _defaultLightFallback;
        private protected bool _isLightFallbackOverridden = false;

        /// <summary>
        /// Gets or sets the solid color to use when system conditions prevent rendering the backdrop material.
        /// </summary>
        /// <remarks>
        /// The backdrop material is replaced with a solid color when one of the fallback conditions is met, such as entering battery saver mode.
        /// </remarks>
        /// <value>The solid color to use when system conditions prevent rendering the backdrop material.</value>
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController.FallbackColor" />
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.FallbackColor" />
        public Windows.UI.Color LightFallbackColor
        {
            get => _lightFallbackColor;
            set
            {
                _isLightFallbackOverridden = true;
                if (_lightFallbackColor != value)
                {
                    _lightFallbackColor = value;
                    NotifyDirty();
                }
            }
        }

        private protected double _darkTintOpacity = 0;
        private protected bool _isDarkTintOpacityOverridden = false;

        /// <summary>
        /// Gets or sets the degree of opacity of the color tint.
        /// </summary>
        /// <value>The degree of opacity of the color tint.</value>
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController.TintOpacity" />
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.TintOpacity" />
        public double DarkTintOpacity
        {
            get => _darkTintOpacity;
            set
            {
                if (value < 0 || value > 1)
                    throw new ArgumentOutOfRangeException(nameof(value));
                _isDarkTintOpacityOverridden = true;
                if (_darkTintOpacity != value)
                {
                    _darkTintOpacity = value;
                    NotifyDirty();
                }
            }
        }

        private protected double _lightTintOpacity = 0;
        private protected bool _isLightTintOpacityOverridden = false;

        /// <summary>
        /// Gets or sets the degree of opacity of the color tint.
        /// </summary>
        /// <value>The degree of opacity of the color tint.</value>
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.MicaController.TintOpacity" />
        /// <seealso cref="Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.TintOpacity" />
        public double LightTintOpacity
        {
            get => _lightTintOpacity;
            set
            {
                if (value < 0 || value > 1)
                    throw new ArgumentOutOfRangeException(nameof(value));
                _isLightTintOpacityOverridden = true;
                if (_lightTintOpacity != value)
                {
                    _lightTintOpacity = value;
                    NotifyDirty();
                }
            }
        }

        private protected double _darkLuminosityOpacity = 1;
        private protected bool _isDarkLuminosityOpacityOverridden = false;

        /// <summary>
        /// Gets or sets the degree of opacity of the color's luminosity.
        /// </summary>
        /// <value>The degree of opacity of the color's luminosity.</value>
        /// <seealso cref="MicaController.LuminosityOpacity" />
        /// <seealso cref="DesktopAcrylicController.LuminosityOpacity" />
        public double DarkLuminosityOpacity
        {
            get => _darkLuminosityOpacity;
            set
            {
                if (value < 0 || value > 1)
                    throw new ArgumentOutOfRangeException(nameof(value));
                _isDarkLuminosityOpacityOverridden = true;
                if (_darkLuminosityOpacity != value)
                {
                    _darkLuminosityOpacity = value;
                    NotifyDirty();
                }
            }
        }

        private protected double _lightLuminosityOpacity = 1;
        private protected bool _isLightLuminosityOpacityOverridden = false;

        /// <summary>
        /// Gets or sets the degree of opacity of the color's luminosity.
        /// </summary>
        /// <value>The degree of opacity of the color's luminosity.</value>
        /// <seealso cref="MicaController.LuminosityOpacity" />
        /// <seealso cref="DesktopAcrylicController.LuminosityOpacity" />
        public double LightLuminosityOpacity
        {
            get => _lightLuminosityOpacity;
            set
            {
                if (value < 0 || value > 1)
                    throw new ArgumentOutOfRangeException(nameof(value));
                _isLightLuminosityOpacityOverridden = true;
                if (_lightLuminosityOpacity != value)
                {
                    _lightLuminosityOpacity = value;
                    NotifyDirty();
                }
            }
        }

        /// <summary>
        /// Notifies the backdrops that properties have changed and the controller needs updating.
        /// </summary>
        /// <seealso cref="UpdateController(ISystemBackdropController, SystemBackdropTheme)"/>
        protected void NotifyDirty()
        {
            IsDirty?.Invoke(this, EventArgs.Empty);
        }

        internal event EventHandler? IsDirty;

        /// <summary>
        /// Creates the backdrop controller.
        /// </summary>
        /// <returns></returns>
        protected internal abstract ISystemBackdropController CreateController();

        /// <summary>
        /// Updates the properties of the backdrop controller.
        /// </summary>
        /// <param name="controller"></param>
        /// <param name="theme"></param>
        protected internal abstract void UpdateController(ISystemBackdropController controller, SystemBackdropTheme theme);

        /// <summary>
        /// Applies the controller to the system backdrop.
        /// </summary>
        /// <param name="controller"></param>
        /// <param name="target"></param>
        /// <param name="configuration"></param>
        /// <remarks>
        /// This usually is a matter of calling <c>AddSystemBackdropTarget</c> and <c>SetSystemBackdropConfiguration</c> on the controller.
        /// Example:
        /// <code lang="cs">
        /// controller.AddSystemBackdropTarget(target);
        /// controller.SetSystemBackdropConfiguration(configuration);
        /// </code>
        /// </remarks>
        /// <seealso cref="MicaController.AddSystemBackdropTarget(ICompositionSupportsSystemBackdrop)"/>
        /// <seealso cref="DesktopAcrylicController.AddSystemBackdropTarget(ICompositionSupportsSystemBackdrop)"/>
        /// <seealso cref="MicaController.SetSystemBackdropConfiguration(SystemBackdropConfiguration)"/>
        /// <seealso cref="DesktopAcrylicController.SetSystemBackdropConfiguration(SystemBackdropConfiguration)"/>
        protected internal abstract void ApplyController(
            ISystemBackdropController controller, Microsoft.UI.Composition.ICompositionSupportsSystemBackdrop target, SystemBackdropConfiguration configuration);

        /// <summary>
        /// Gets a proper indicating whether the system supports this backdrop.
        /// </summary>
        public abstract bool IsSupported { get; }
    }

    /// <summary>
    /// Defines the Mica System Backdrop settings to apply to the window.
    /// Note: requires Windows 11 and up - Windows 10 will use fallback colors.
    /// </summary>
    /// <seealso cref="AcrylicSystemBackdrop"/>
    [Obsolete("Use Microsoft.UI.Xaml.Media.MicaBackdrop")]
    [EditorBrowsable(EditorBrowsableState.Never)]
    public class MicaSystemBackdrop : SystemBackdrop
    {
        private readonly static Windows.UI.Color _defaultDarkTint = Windows.UI.Color.FromArgb(0xff, 0x20, 0x20, 0x20);
        private readonly static Windows.UI.Color _defaultLightTint = Windows.UI.Color.FromArgb(0xff, 0xf3, 0xf3, 0xf3);
        private readonly static Windows.UI.Color _altDarkTint = Windows.UI.Color.FromArgb(0xff, 0x0a, 0x0a, 0x0a);
        private readonly static Windows.UI.Color _altLightTint = Windows.UI.Color.FromArgb(0xff, 0xda, 0xda, 0xda);
        private readonly static Windows.UI.Color _altDarkFallback = Windows.UI.Color.FromArgb(0xff, 0x20, 0x20, 0x20);
        private readonly static Windows.UI.Color _altLightFallback = Windows.UI.Color.FromArgb(0xff, 0xe8, 0xe8, 0xe8);

        /// <summary>
        /// Initializes a new instance of the <see cref="MicaSystemBackdrop"/> class.
        /// </summary>
        public MicaSystemBackdrop()
        {
            ResetProperties();
        }

        /// <inheritdoc />
        public override void ResetProperties()
        {
            base.ResetProperties();
            _lightTintOpacity = 0.5;
            _darkTintOpacity = _kind == MicaKind.Base ? 0.8 : 0;
            _darkTintColor = _kind == MicaKind.Base ? _defaultDarkTint : _altDarkTint;
            _lightTintColor = _kind == MicaKind.Base ? _defaultLightTint : _altLightTint;
            _darkFallbackColor = _kind == MicaKind.Base ? _defaultDarkFallback : _altDarkFallback;
            _lightFallbackColor = _kind == MicaKind.Base ? _defaultLightFallback : _altLightFallback;
            NotifyDirty();
        }

        private MicaKind _kind;

        /// <summary>
        /// Gets or sets the Mica backdrop kind.
        /// </summary>
        /// <remarks>
        /// Setting the Kind property will change the Tint and Fallback colors to new defaults. Note that if you
        /// explicitly set the Tint and Fallback colors, changing the Kind will have no effect. This matches the
        /// MicaController behavior in the Windows App SDK.
        /// </remarks>
        /// <seealso cref="MicaController.Kind"/>
        public MicaKind Kind
        {
            get => _kind;
            set {
                _kind = value;
                // Only overwrite defaults if they haven't been explicitly set. These 5 values change when Kind is set
                if(!_isDarkTintOverridden)
                     _darkTintColor = _kind == MicaKind.Base ? _defaultDarkTint : _altDarkTint;
                if (!_isLightTintOverridden)
                    _lightTintColor = _kind == MicaKind.Base ? _defaultLightTint : _altLightTint;
                if (!_isDarkFallbackOverridden)
                    _darkFallbackColor = _kind == MicaKind.Base ? _defaultDarkFallback : _altDarkFallback;
                if (!_isLightFallbackOverridden)
                    _lightFallbackColor = _kind == MicaKind.Base ? _defaultLightFallback : _altLightFallback;
                if (!_isDarkTintOpacityOverridden)
                    _darkTintOpacity = _kind == MicaKind.Base ? 0.8 : 0;
                NotifyDirty(); }
        }

        /// <inheritdoc/>
        public override bool IsSupported => MicaController.IsSupported();

        /// <inheritdoc/>
        protected internal override ISystemBackdropController CreateController() => new MicaController();
        
        /// <inheritdoc/>
        protected internal override void UpdateController(ISystemBackdropController controller, SystemBackdropTheme theme)
        {
            if (controller is MicaController mica)
            {
                bool isDark = theme == SystemBackdropTheme.Dark;
                mica.ResetProperties();
                mica.Kind = Kind;
                if (isDark && _isDarkTintOpacityOverridden || !isDark && _isLightTintOpacityOverridden)
                    mica.TintOpacity = (float)(isDark ? DarkTintOpacity : LightTintOpacity);
                if (isDark && _isDarkLuminosityOpacityOverridden || !isDark && _isLightLuminosityOpacityOverridden)
                    mica.LuminosityOpacity = (float)(isDark ? DarkLuminosityOpacity : LightLuminosityOpacity);
                if (isDark && _isDarkTintOverridden || !isDark && _isLightTintOverridden)
                    mica.TintColor = isDark ? DarkTintColor : LightTintColor;
                if (isDark && _isDarkFallbackOverridden || !isDark && _isLightFallbackOverridden)
                    mica.FallbackColor = isDark ? DarkFallbackColor : LightFallbackColor;
            }
        }

        /// <inheritdoc/>
        protected internal override void ApplyController(ISystemBackdropController controller, ICompositionSupportsSystemBackdrop target, SystemBackdropConfiguration configuration)
        {
            if (controller is MicaController mica)
            {
                mica.AddSystemBackdropTarget(target);
                mica.SetSystemBackdropConfiguration(configuration);
            }
        }
    }

    /// <summary>
    /// Defines the Acrylic System Backdrop settings to apply to the window.
    /// </summary>
    /// <seealso cref="DesktopAcrylicController"/>
    /// <seealso cref="MicaSystemBackdrop"/>
    [Obsolete("Use Microsoft.UI.Xaml.Media.DesktopAcrylicBackdrop")]
    [EditorBrowsable(EditorBrowsableState.Never)]
    public class AcrylicSystemBackdrop : SystemBackdrop
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="AcrylicSystemBackdrop"/> class.
        /// </summary>
        public AcrylicSystemBackdrop()
        {
            ResetProperties();
        }

        /// <inheritdoc />
        public override void ResetProperties()
        {
            base.ResetProperties();
            _lightTintColor = Windows.UI.Color.FromArgb(0xff, 0xd3, 0xd3, 0xd3);
            _lightFallbackColor = Windows.UI.Color.FromArgb(0xff, 0xd3, 0xd3, 0xd3);
            _lightLuminosityOpacity = .64;
            _darkTintColor = Windows.UI.Color.FromArgb(0xff, 0x54, 0x54, 0x54);
            _darkFallbackColor = Windows.UI.Color.FromArgb(0xff, 0x54, 0x54, 0x54);
            _darkLuminosityOpacity = .64;
            NotifyDirty();
        }

        /// <inheritdoc/>
        public override bool IsSupported => DesktopAcrylicController.IsSupported();

        /// <inheritdoc/>
        protected internal override ISystemBackdropController CreateController() => new DesktopAcrylicController();

        /// <inheritdoc/>
        protected internal override void UpdateController(ISystemBackdropController controller, SystemBackdropTheme theme)
        {
            if (controller is DesktopAcrylicController acrylic)
            {
                bool isDark = theme == SystemBackdropTheme.Dark;
                acrylic.ResetProperties();
                if (isDark && _isDarkTintOpacityOverridden || !isDark && _isLightTintOpacityOverridden)
                    acrylic.TintOpacity = (float)(isDark ? DarkTintOpacity : LightTintOpacity);
                if (isDark && _isDarkLuminosityOpacityOverridden || !isDark && _isLightLuminosityOpacityOverridden)
                    acrylic.LuminosityOpacity = (float)(isDark ? DarkLuminosityOpacity : LightLuminosityOpacity);
                if (isDark && _isDarkTintOverridden || !isDark && _isLightTintOverridden)
                    acrylic.TintColor = isDark ? DarkTintColor : LightTintColor;
                if (isDark && _isDarkFallbackOverridden || !isDark && _isLightFallbackOverridden)
                    acrylic.FallbackColor = isDark ? DarkFallbackColor : LightFallbackColor;
            }
        }

        /// <inheritdoc/>
        protected internal override void ApplyController(ISystemBackdropController controller, ICompositionSupportsSystemBackdrop target, SystemBackdropConfiguration configuration)
        {
            if (controller is DesktopAcrylicController acrylic)
            {
                acrylic.AddSystemBackdropTarget(target);
                acrylic.SetSystemBackdropConfiguration(configuration);
            }
        }
    }
}
