using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Sucrose.Portal.Helpers;
using System.Runtime.InteropServices;
using Windows.UI.ViewManagement;
using WinRT;
using WinRT.Interop;

namespace Sucrose.Portal;

public class WindowsSystemDispatcherQueueHelper
{
    [StructLayout(LayoutKind.Sequential)]
    struct DispatcherQueueOptions
    {
        internal int dwSize;
        internal int threadType;
        internal int apartmentType;
    }

    [DllImport("CoreMessaging.dll")]
    private static unsafe extern int CreateDispatcherQueueController(DispatcherQueueOptions options, IntPtr* instance);

    IntPtr m_dispatcherQueueController = IntPtr.Zero;
    public void EnsureWindowsSystemDispatcherQueueController()
    {
        if (Windows.System.DispatcherQueue.GetForCurrentThread() != null)
        {
            // one already exists, so we'll just use it.
            return;
        }

        if (m_dispatcherQueueController == IntPtr.Zero)
        {
            DispatcherQueueOptions options;
            options.dwSize = Marshal.SizeOf(typeof(DispatcherQueueOptions));
            options.threadType = 2;    // DQTYPE_THREAD_CURRENT
            options.apartmentType = 2; // DQTAT_COM_STA

            unsafe
            {
                IntPtr dispatcherQueueController;
                CreateDispatcherQueueController(options, &dispatcherQueueController);
                m_dispatcherQueueController = dispatcherQueueController;
            }
        }
    }
}

public sealed partial class MainWindow : WindowEx
{
    private DispatcherQueue dispatcherQueue;

    private BackdropType m_currentBackdrop;
    private WindowsSystemDispatcherQueueHelper m_wsdqHelper;
    private Microsoft.UI.Composition.SystemBackdrops.MicaController m_micaController;
    private Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController m_acrylicController;
    private Microsoft.UI.Composition.SystemBackdrops.SystemBackdropConfiguration m_configurationSource;

    private UISettings settings;

    public MainWindow()
    {
        InitializeComponent();

        AppWindow.SetIcon(Path.Combine(AppContext.BaseDirectory, "Assets/ICO.ico"));

        Content = null;

        Title = "AppDisplayName".GetLocalized();

        // Theme change code picked from https://github.com/microsoft/WinUI-Gallery/pull/1239
        dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        settings = new UISettings();
        settings.ColorValuesChanged += Settings_ColorValuesChanged; // cannot use FrameworkElement.ActualThemeChanged event

        this.CenterOnScreen();

        ExtendsContentIntoTitleBar = true;

        m_wsdqHelper = new WindowsSystemDispatcherQueueHelper();
        m_wsdqHelper.EnsureWindowsSystemDispatcherQueueController();

        //SetBackdrop(BackdropType.DesktopAcrylicThin);
    }

    // this handles updating the caption button colors correctly when indows system theme is changed
    // while the app is open
    private void Settings_ColorValuesChanged(UISettings sender, object args)
    {
        // This calls comes off-thread, hence we will need to dispatch it to current app's thread
        dispatcherQueue.TryEnqueue(TitleBarHelper.ApplySystemThemeToCaptionButtons);
    }

    public enum BackdropType
    {
        None,
        Mica,
        MicaAlt,
        DesktopAcrylic,
        DesktopAcrylicBase,
        DesktopAcrylicThin
    }

    public void SetBackdrop(BackdropType type)
    {
        // Reset to default color. If the requested type is supported, we'll update to that.
        // Note: This sample completely removes any previous controller to reset to the default
        //       state. This is done so this sample can show what is expected to be the most
        //       common pattern of an app simply choosing one controller type which it sets at
        //       startup. If an app wants to toggle between Mica and Acrylic it could simply
        //       call RemoveSystemBackdropTarget() on the old controller and then setup the new
        //       controller, reusing any existing m_configurationSource and Activated/Closed
        //       event handlers.
        m_currentBackdrop = BackdropType.None;

        this.SystemBackdrop = null;

        if (type == BackdropType.Mica)
        {
            if (TrySetMicaBackdrop1(false))
            {
                m_currentBackdrop = type;
            }
            else
            {
                // Mica isn't supported. Try Acrylic.
                type = BackdropType.DesktopAcrylic;
            }
        }
        if (type == BackdropType.MicaAlt)
        {
            if (TrySetMicaBackdrop1(true))
            {
                m_currentBackdrop = type;
            }
            else
            {
                // MicaAlt isn't supported. Try Acrylic.
                type = BackdropType.DesktopAcrylic;
            }
        }
        if (type == BackdropType.DesktopAcrylicBase)
        {
            if (TrySetAcrylicBackdrop(false))
            {
                m_currentBackdrop = type;
            }
            else
            {
                // Acrylic isn't supported, so take the next option, which is DefaultColor, which is already set.
            }
        }
        if (type == BackdropType.DesktopAcrylicThin)
        {
            if (TrySetAcrylicBackdrop(true))
            {
                m_currentBackdrop = type;
            }
            else
            {
                // Acrylic isn't supported, so take the next option, which is DefaultColor, which is already set.
            }
        }
        if (type == BackdropType.DesktopAcrylic)
        {
            if (TrySetAcrylicBackdrop())
            {
                m_currentBackdrop = type;
            }
            else
            {
                // Acrylic isn't supported, so take the next option, which is DefaultColor, which is already set.
            }
        }
    }

    bool TrySetMicaBackdrop1(bool useMicaAlt)
    {
        if (Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported())
        {
            Microsoft.UI.Xaml.Media.MicaBackdrop micaBackdrop = new Microsoft.UI.Xaml.Media.MicaBackdrop();
            micaBackdrop.Kind = useMicaAlt ? Microsoft.UI.Composition.SystemBackdrops.MicaKind.BaseAlt : Microsoft.UI.Composition.SystemBackdrops.MicaKind.Base;
            this.SystemBackdrop = micaBackdrop;
            return true;
        }

        return false; // Mica is not supported on this system
    }

    bool TrySetAcrylicBackdrop()
    {
        if (Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.IsSupported())
        {
            this.SystemBackdrop = new Microsoft.UI.Xaml.Media.DesktopAcrylicBackdrop();
            return true;
        }

        return false; // Acrylic is not supported on this system
    }

    bool TrySetMicaBackdrop(bool useMicaAlt)
    {
        if (Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported())
        {
            // Hooking up the policy object.
            m_configurationSource = new Microsoft.UI.Composition.SystemBackdrops.SystemBackdropConfiguration();
            this.Activated += Window_Activated;
            this.Closed += Window_Closed;

            ((FrameworkElement)this.Content).ActualThemeChanged += Window_ThemeChanged;

            // Initial configuration state.
            m_configurationSource.IsInputActive = true;
            SetConfigurationSourceTheme();

            m_micaController = new Microsoft.UI.Composition.SystemBackdrops.MicaController();

            m_micaController.Kind = useMicaAlt ? Microsoft.UI.Composition.SystemBackdrops.MicaKind.BaseAlt : Microsoft.UI.Composition.SystemBackdrops.MicaKind.Base;

            // Enable the system backdrop.
            // Note: Be sure to have "using WinRT;" to support the Window.As<...>() call.
            m_micaController.AddSystemBackdropTarget(this.As<Microsoft.UI.Composition.ICompositionSupportsSystemBackdrop>());
            m_micaController.SetSystemBackdropConfiguration(m_configurationSource);
            return true; // Succeeded.
        }

        return false; // Mica is not supported on this system.
    }

    bool TrySetAcrylicBackdrop(bool useAcrylicThin)
    {
        if (Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController.IsSupported())
        {
            // Hooking up the policy object.
            m_configurationSource = new Microsoft.UI.Composition.SystemBackdrops.SystemBackdropConfiguration();
            this.Activated += Window_Activated;
            this.Closed += Window_Closed;

            ((FrameworkElement)this.Content).ActualThemeChanged += Window_ThemeChanged;

            // Initial configuration state.
            m_configurationSource.IsInputActive = true;
            SetConfigurationSourceTheme();

            m_acrylicController = new Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicController();

            m_acrylicController.Kind = useAcrylicThin ? Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicKind.Thin : Microsoft.UI.Composition.SystemBackdrops.DesktopAcrylicKind.Base;

            // Enable the system backdrop.
            // Note: Be sure to have "using WinRT;" to support the Window.As<...>() call.
            m_acrylicController.AddSystemBackdropTarget(this.As<Microsoft.UI.Composition.ICompositionSupportsSystemBackdrop>());
            m_acrylicController.SetSystemBackdropConfiguration(m_configurationSource);
            
            return true; // Succeeded.
        }

        return false; // Acrylic is not supported on this system
    }

    private void Window_Activated(object sender, WindowActivatedEventArgs args)
    {
        m_configurationSource.IsInputActive = args.WindowActivationState != WindowActivationState.Deactivated;
    }

    private void Window_Closed(object sender, WindowEventArgs args)
    {
        // Make sure any Mica/Acrylic controller is disposed so it doesn't try to
        // use this closed window.
        if (m_micaController != null)
        {
            m_micaController.Dispose();
            m_micaController = null;
        }
        if (m_acrylicController != null)
        {
            m_acrylicController.Dispose();
            m_acrylicController = null;
        }
        this.Activated -= Window_Activated;
        m_configurationSource = null;
    }

    private void Window_ThemeChanged(FrameworkElement sender, object args)
    {
        if (m_configurationSource != null)
        {
            SetConfigurationSourceTheme();
        }
    }

    private void SetConfigurationSourceTheme()
    {
        //switch (((FrameworkElement)this.Content).ActualTheme)
        //{
        //    case ElementTheme.Dark: m_configurationSource.Theme = Microsoft.UI.Composition.SystemBackdrops.SystemBackdropTheme.Dark; break;
        //    case ElementTheme.Light: m_configurationSource.Theme = Microsoft.UI.Composition.SystemBackdrops.SystemBackdropTheme.Light; break;
        //    case ElementTheme.Default: m_configurationSource.Theme = Microsoft.UI.Composition.SystemBackdrops.SystemBackdropTheme.Default; break;
        //}
    }
}