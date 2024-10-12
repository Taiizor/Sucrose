using System.Windows;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSLHC = Sucrose.Shared.Launcher.Helper.Create;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;

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
        }

        private async void CreateButton_Click(object sender, RoutedEventArgs e)
        {
            Create_Button.IsEnabled = false;

            await Task.Run(SSLHC.Start);

            Create_Button.IsEnabled = true;
        }

        private void ReportButton_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Report}{SMMRG.ValueSeparator}{SMMRU.GitHubSucroseReport}");
        }
    }
}