using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Controls;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// OtherHelp.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherHelp : ContentDialog, IDisposable
    {
        public OtherHelp(ContentPresenter? contentPresenter) : base(contentPresenter)
        {
            InitializeComponent();
        }

        private void Report_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Report}{SMMRG.ValueSeparator}{SMMRU.GitHubSucroseReport}");
        }

        private void WebPage_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Official}{SMMRG.ValueSeparator}{SMMRU.OfficialWebPage}");
        }

        private void Repository_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Repository}{SMMRG.ValueSeparator}{SMMRU.GitHubSucrose}");
        }

        private void Discussions_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Discussions}{SMMRG.ValueSeparator}{SMMRU.GitHubSucroseDiscussions}");
        }

        private void Documentation_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Wiki}{SMMRG.ValueSeparator}{SMMRU.GitHubSucroseWiki}");
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