using System.Windows;
using SGHLL = Sucrose.Globalization.Helper.LauncherLocalization;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSLHC = Sucrose.Shared.Launcher.Helper.Create;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Launcher.View
{
    /// <summary>
    /// Interaction logic for LightReportBox.xaml
    /// </summary>
    public partial class LightReportBox : Window
    {
        public LightReportBox()
        {
            InitializeComponent();

            Title = SGHLL.GetValue("ReportWindowTitle");
            Create_Button.Content = SGHLL.GetValue("CreateLogText");
            Report_Button.Content = SGHLL.GetValue("OpenReportText");
        }

        private void CreateButton_Click(object sender, RoutedEventArgs e)
        {
            SSLHC.Start();
        }

        private void ReportButton_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }
    }
}