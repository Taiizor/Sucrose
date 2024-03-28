using Sucrose.Shared.Space.Helper;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Input;
using Wpf.Ui.Appearance;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;
using SWMM = Sucrose.Wizard.Manage.Manager;

namespace Sucrose.Wizard.View
{
    /// <summary>
    /// MainWindow.xaml etkileşim mantığı
    /// </summary>
    public partial class MainWindow : Window
    {
        private static string Bundle { get; set; } = string.Empty;

        private static bool HasBundle { get; set; } = false;

        private static bool HasError { get; set; } = true;

        private static bool HasInfo { get; set; } = false;

        private static bool HasFile { get; set; } = false;

        private static int MinDelay => 1000;

        public MainWindow()
        {
            InitializeComponent();
        }

        private void WindowCorner()
        {
            try
            {
                SWNM.DWMWINDOWATTRIBUTE Attribute = SWNM.DWMWINDOWATTRIBUTE.WindowCornerPreference;
                SWNM.DWM_WINDOW_CORNER_PREFERENCE Preference = SWNM.DWM_WINDOW_CORNER_PREFERENCE.DWMWCP_ROUND;

                SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), Attribute, ref Preference, (uint)Marshal.SizeOf(typeof(uint)));
            }
            catch
            {
                //
            }
        }

        private void Window_ContentRendered(object sender, EventArgs e)
        {
            WindowCorner();

            Message.Text = "CREATING TEMPORARY LOCATIONS";

            Message.Text = "CHECKING INTERNET CONNECTION";
            Message.Text = "NO INTERNET CONNECTION";

            Message.Text = "GETTING NECESSARY LISTS";
            Message.Text = "UNABLE TO GET NECESSARY LISTS";
            Message.Text = "NECESSARY LISTS ARE EMPTY";
        }

        private void Window_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ChangedButton == MouseButton.Left)
            {
                Cursor = Cursors.SizeAll;
                DragMove();
                Cursor = Cursors.Arrow;
            }
        }
    }
}
