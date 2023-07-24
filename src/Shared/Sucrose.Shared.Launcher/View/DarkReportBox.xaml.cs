using System.Runtime.InteropServices;
using System.Windows;
using SGHLL = Sucrose.Globalization.Helper.LauncherLocalization;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSLHC = Sucrose.Shared.Launcher.Helper.Create;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Launcher.View
{
    /// <summary>
    /// Interaction logic for DarkReportBox.xaml
    /// </summary>
    public partial class DarkReportBox : Window
    {
        public DarkReportBox()
        {
            InitializeComponent();

            Title = SGHLL.GetValue("ReportWindowTitle");
            Create_Button.Content = SGHLL.GetValue("CreateLogText");
            Report_Button.Content = SGHLL.GetValue("OpenReportText");

            SourceInitialized += DarkReportBox_SourceInitialized;
        }

        private void CreateButton_Click(object sender, RoutedEventArgs e)
        {
            SSLHC.Start();
        }

        private void ReportButton_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }

        private void DarkReportBox_SourceInitialized(object sender, EventArgs e)
        {
            bool Value = true;

            SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), SWNM.DWMWindowAttribute.DWMWA_USE_IMMERSIVE_DARK_MODE, ref Value, Marshal.SizeOf(Value));
        }
    }
}