using System.Windows;
using Wpf.Ui.Controls;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// OtherHelp.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherHelp : ContentDialog, IDisposable
    {
        public OtherHelp() : base(SPMI.ContentDialogService.GetContentPresenter())
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
            WebPageExpander.TitleText = SRER.GetValue("Portal", "OtherHelp", "WebPage");
            WebPageExpander.DescriptionText = SRER.GetValue("Portal", "OtherHelp", "WebPage", "Description");

            DocumentationExpander.TitleText = SRER.GetValue("Portal", "OtherHelp", "Documentation");
            DocumentationExpander.DescriptionText = SRER.GetValue("Portal", "OtherHelp", "Documentation", "Description");

            RepositoryExpander.TitleText = SRER.GetValue("Portal", "OtherHelp", "Repository");
            RepositoryExpander.DescriptionText = SRER.GetValue("Portal", "OtherHelp", "Repository", "Description");

            DiscussionsExpander.TitleText = SRER.GetValue("Portal", "OtherHelp", "Discussions");
            DiscussionsExpander.DescriptionText = SRER.GetValue("Portal", "OtherHelp", "Discussions", "Description");

            ReportExpander.TitleText = SRER.GetValue("Portal", "OtherHelp", "Report");
            ReportExpander.DescriptionText = SRER.GetValue("Portal", "OtherHelp", "Report", "Description");
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}