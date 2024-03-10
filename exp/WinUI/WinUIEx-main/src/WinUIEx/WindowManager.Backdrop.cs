using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using System;
using System.Runtime.InteropServices;
using Windows.Storage;
using WinUIEx.Messaging;
using Windows.Win32.UI.WindowsAndMessaging;
using Microsoft.UI.Composition.SystemBackdrops;
using WinRT;
using Windows.UI;
using System.ComponentModel;

namespace WinUIEx
{
    /// <summary>
    /// Manages Window size, and ensures window correctly resizes during DPI changes to keep consistent
    /// DPI-independent sizing.
    /// </summary>
    public partial class WindowManager : IDisposable
    {
        private static IntPtr m_dispatcherQueueController = IntPtr.Zero;
        private ISystemBackdropController? currentController;
        private SystemBackdropConfiguration? BackdropConfiguration;
        [Obsolete]
        private SystemBackdrop? m_backdrop;

        /// <summary>
        /// Gets or sets the system backdrop for the window.
        /// Note: Windows 10 doesn't support Acrylic, so will fall back to default backdrop.
        /// </summary>
        /// <seealso cref="MicaSystemBackdrop"/>
        /// <seealso cref="AcrylicSystemBackdrop"/>
        [Obsolete("Use Microsoft.UI.Xaml.Window.SystemBackdrop")]
        [EditorBrowsable(EditorBrowsableState.Never)]
        public SystemBackdrop? Backdrop
        {
            get => m_backdrop;
            set
            {
                if (m_backdrop != value)
                {
                    if (m_backdrop != null)
                        m_backdrop.IsDirty -= Backdrop_IsDirty;
                    m_backdrop = value;
                    CleanUpBackdrop();
                    if (m_backdrop != null)
                        m_backdrop.IsDirty += Backdrop_IsDirty;
                    if (m_backdrop is not null && _window.Visible)
                        InitBackdrop();
                }
            }
        }

        private static bool IsEmpty(Windows.UI.Color c) => c.A == 0 && c.R == 0 && c.G == 0 && c.B == 0;

        private void Backdrop_IsDirty(object? sender, EventArgs e)
        {
#pragma warning disable CS0612 // Type or member is obsolete
            if (currentController != null && BackdropConfiguration != null && m_backdrop != null)
                m_backdrop.UpdateController(currentController, BackdropConfiguration.Theme);
#pragma warning restore CS0612 // Type or member is obsolete
        }

        /// <summary>
        /// Gets the currently active window backdrop.
        /// </summary>
        [EditorBrowsable(EditorBrowsableState.Never)]
        [Obsolete()]
        public ISystemBackdropController? ActiveBackdropController => currentController;

        private void InitBackdrop()
        {
#pragma warning disable CS0612 // Type or member is obsolete
            if (m_backdrop is null)
            {
                CleanUpBackdrop();
                return;
            }

            if (BackdropConfiguration is null)
            {
                EnsureDispatcherQueueController();
                BackdropConfiguration = new SystemBackdropConfiguration();

                var rootElement = _window.Content as FrameworkElement;
                if (rootElement is not null)
                {
                    // This should probably be weak in the rare event the root content changes
                    // Unfortunately there's no good event to detect changes though.
                    rootElement.ActualThemeChanged += (s, e) =>
                    {
                        if (BackdropConfiguration != null && currentController != null)
                        {
                            BackdropConfiguration.Theme = ConvertToSystemBackdropTheme(s.ActualTheme);
                            m_backdrop.UpdateController(currentController, BackdropConfiguration.Theme);
                        }
                    };

                    // Initial state.
                    BackdropConfiguration.Theme = ConvertToSystemBackdropTheme(rootElement.ActualTheme);
                }
            }
            if (currentController != null)
            {
                currentController.Dispose();
                currentController = null;
            }
            var controller = m_backdrop.CreateController();
            m_backdrop.UpdateController(controller, BackdropConfiguration.Theme);
            m_backdrop.ApplyController(controller, _window.As<Microsoft.UI.Composition.ICompositionSupportsSystemBackdrop>(), BackdropConfiguration);
            currentController = controller;
#pragma warning restore CS0612 // Type or member is obsolete
        }

        private void CleanUpBackdrop()
        {
            (currentController as MicaController)?.RemoveAllSystemBackdropTargets();
            (currentController as DesktopAcrylicController)?.RemoveAllSystemBackdropTargets();
            currentController?.Dispose();
            currentController = null;
        }

        private static void EnsureDispatcherQueueController()
        {
            if (Windows.System.DispatcherQueue.GetForCurrentThread() == null && m_dispatcherQueueController == IntPtr.Zero)
            {
                DispatcherQueueOptions options;
                options.dwSize = Marshal.SizeOf(typeof(DispatcherQueueOptions));
                options.threadType = 2;    // DQTYPE_THREAD_CURRENT
                options.apartmentType = 2; // DQTAT_COM_STA

                CreateDispatcherQueueController(options, out m_dispatcherQueueController);
            }
        }
        static Windows.UI.Composition.Compositor? compositor;
        static object compositorLock = new object();
        internal static Windows.UI.Composition.Compositor Compositor
        {
            get
            {
                if (compositor == null)
                {
                    lock (compositorLock)
                    {
                        if (compositor == null)
                        {
                            EnsureDispatcherQueueController();
                            compositor = new Windows.UI.Composition.Compositor();
                        }
                    }
                }
                return compositor;
            }
        }
        private static SystemBackdropTheme ConvertToSystemBackdropTheme(ElementTheme theme)
        {
            switch (theme)
            {
                case ElementTheme.Dark:
                    return SystemBackdropTheme.Dark;
                case ElementTheme.Light:
                    return SystemBackdropTheme.Light;
                default:
                    return SystemBackdropTheme.Default;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct DispatcherQueueOptions
        {
            internal int dwSize;
            internal int threadType;
            internal int apartmentType;
        }

        [DllImport("CoreMessaging.dll")]
        private static extern int CreateDispatcherQueueController([In] DispatcherQueueOptions options, out IntPtr dispatcherQueueController);
    }
}
