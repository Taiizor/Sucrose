using System.Windows;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSLHC = Sucrose.Shared.Launcher.Helper.Create;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;

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

            SourceInitialized += DarkReportBox_SourceInitialized;
        }

        private async void CreateButton_Click(object sender, RoutedEventArgs e)
        {
            Create_Button.IsEnabled = false;

            await Task.Run(SSLHC.Start);

            Create_Button.IsEnabled = true;
        }

        private void ReportButton_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMMRU.GitHubSucroseReport}");
        }

        private void DarkReportBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }
    }
}