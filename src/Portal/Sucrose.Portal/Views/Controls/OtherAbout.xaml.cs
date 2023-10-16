using System.Windows;
using Wpf.Ui.Controls;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// OtherAbout.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherAbout : ContentDialog, IDisposable
    {
        public OtherAbout() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void Report_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }

        private void WebPage_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Official}{SMR.ValueSeparator}{SMR.OfficialWebsite}");
        }

        private void Repository_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Repository}{SMR.ValueSeparator}{SMR.RepositoryWebsite}");
        }

        private void Discussions_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Discussions}{SMR.ValueSeparator}{SMR.DiscussionsWebsite}");
        }

        private void Documentation_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Wiki}{SMR.ValueSeparator}{SMR.WikiWebsite}");
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            WebPage.TitleText = SSRER.GetValue("Portal", "OtherHelp", "WebPage");
            WebPage.DescriptionText = SSRER.GetValue("Portal", "OtherHelp", "WebPage", "Description");

            Documentation.TitleText = SSRER.GetValue("Portal", "OtherHelp", "Documentation");
            Documentation.DescriptionText = SSRER.GetValue("Portal", "OtherHelp", "Documentation", "Description");

            Repository.TitleText = SSRER.GetValue("Portal", "OtherHelp", "Repository");
            Repository.DescriptionText = SSRER.GetValue("Portal", "OtherHelp", "Repository", "Description");

            Discussions.TitleText = SSRER.GetValue("Portal", "OtherHelp", "Discussions");
            Discussions.DescriptionText = SSRER.GetValue("Portal", "OtherHelp", "Discussions", "Description");

            Report.TitleText = SSRER.GetValue("Portal", "OtherHelp", "Report");
            Report.DescriptionText = SSRER.GetValue("Portal", "OtherHelp", "Report", "Description");
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}