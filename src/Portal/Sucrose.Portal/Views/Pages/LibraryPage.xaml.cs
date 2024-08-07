using System.IO;
using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SHG = Skylark.Helper.Generator;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCTI = Sucrose.Portal.Views.Controls.ThemeImport;
using SPVMPLVM = Sucrose.Portal.ViewModels.Pages.LibraryViewModel;
using SPVPLELP = Sucrose.Portal.Views.Pages.Library.EmptyLibraryPage;
using SPVPLFLP = Sucrose.Portal.Views.Pages.Library.FullLibraryPage;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;
using SSZHA = Sucrose.Shared.Zip.Helper.Archive;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// LibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryPage : INavigableView<SPVMPLVM>, IDisposable
    {
        private Dictionary<string, string> Searches = new();

        private static List<string> Themes = SMMM.Themes;

        private SPVPLELP EmptyLibraryPage { get; set; }

        private SPVPLFLP FullLibraryPage { get; set; }

        public SPVMPLVM ViewModel { get; }

        public LibraryPage(SPVMPLVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            CheckShowcase();

            InitializeThemes();

            InitializeComponent();

            Library();

            Search();
        }

        private void Search()
        {
            string Search = SPMI.SearchService.SearchText;

            SPMI.SearchService.Dispose();

            SPMI.SearchService = new()
            {
                SearchText = Search
            };

            SPMI.SearchService.SearchTextChanged += SearchService_SearchTextChanged;
        }

        private void Library()
        {
            SPMI.LibraryService.Dispose();

            SPMI.LibraryService = new();

            SPMI.LibraryService.CreatedWallpaper += LibraryService_CreatedWallpaper;
        }

        private void CheckShowcase()
        {
            string ShowcasePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.Showcase);

            if (Directory.Exists(ShowcasePath))
            {
                bool State = false;

                List<string> Showcase = SMMM.Showcase;

                if (!Directory.Exists(SMMM.LibraryLocation))
                {
                    Directory.CreateDirectory(SMMM.LibraryLocation);
                }

                foreach (string Directory in Directory.GetDirectories(ShowcasePath, "*", SearchOption.TopDirectoryOnly).Where(Directory => !Showcase.Contains(Path.GetFileName(Directory))))
                {
                    SSSHC.Folder(Directory, Path.Combine(SMMM.LibraryLocation, Path.GetFileName(Directory)), false);

                    Showcase.Add(Path.GetFileName(Directory));

                    State = true;
                }

                if (State)
                {
                    SMMI.UserSettingManager.SetSetting(SMC.Showcase, Showcase);
                }
            }
        }

        private void InitializeThemes()
        {
            if (Searches != null && Searches.Any())
            {
                Searches.Clear();
            }

            if (Directory.Exists(SMMM.LibraryLocation))
            {
                if (Themes != null && Themes.Any())
                {
                    foreach (string Theme in Themes.ToList())
                    {
                        string PropertiesCache = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Properties);
                        string PropertiesFile = Path.Combine(PropertiesCache, $"{Theme}.json");
                        string ThemePath = Path.Combine(SMMM.LibraryLocation, Theme);
                        string InfoPath = Path.Combine(ThemePath, SMR.SucroseInfo);

                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                        {
                            string InfoContent = SSTHI.ReadInfo(InfoPath);

                            if (SSTHI.CheckJson(InfoContent))
                            {
                                SSTHI Info = SSTHI.FromJson(InfoContent);

                                string PreviewPath = Path.Combine(ThemePath, Info.Preview);
                                string ThumbnailPath = Path.Combine(ThemePath, Info.Thumbnail);

                                if (File.Exists(PreviewPath) && File.Exists(ThumbnailPath) && (SSTHV.IsUrl(Info.Source) || File.Exists(Path.Combine(ThemePath, Info.Source))))
                                {
                                    continue;
                                }
                            }
                        }

                        Themes.Remove(Theme);

                        if (Directory.Exists(ThemePath) && SMMM.LibraryDelete && SSSHA.Directory(ThemePath))
                        {
                            Directory.Delete(ThemePath, true);

                            if (File.Exists(PropertiesFile))
                            {
                                File.Delete(PropertiesFile);
                            }
                        }
                    }
                }

                string[] Folders = Directory.GetDirectories(SMMM.LibraryLocation);

                if (Folders != null && Folders.Any())
                {
                    foreach (string Folder in Folders)
                    {
                        string PropertiesCache = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Properties);
                        string PropertiesFile = Path.Combine(PropertiesCache, $"{Path.GetFileName(Folder)}.json");
                        string InfoPath = Path.Combine(Folder, SMR.SucroseInfo);

                        if (File.Exists(InfoPath))
                        {
                            string InfoContent = SSTHI.ReadInfo(InfoPath);

                            if (SSTHI.CheckJson(InfoContent))
                            {
                                SSTHI Info = SSTHI.FromJson(InfoContent);

                                string PreviewPath = Path.Combine(Folder, Info.Preview);
                                string ThumbnailPath = Path.Combine(Folder, Info.Thumbnail);

                                if (File.Exists(PreviewPath) && File.Exists(ThumbnailPath) && !Themes.Contains(Path.GetFileName(Folder)) && (SSTHV.IsUrl(Info.Source) || File.Exists(Path.Combine(Folder, Info.Source))))
                                {
                                    Themes.Add(Path.GetFileName(Folder));
                                    continue;
                                }
                            }
                        }

                        if (SMMM.LibraryDelete && SSSHA.Directory(Folder))
                        {
                            Directory.Delete(Folder, true);

                            if (File.Exists(PropertiesFile))
                            {
                                File.Delete(PropertiesFile);
                            }
                        }
                    }
                }

                if (Themes != null && Themes.Any())
                {
                    Dictionary<string, object> SortThemes = new();

                    foreach (string Theme in Themes.ToList())
                    {
                        string ThemePath = Path.Combine(SMMM.LibraryLocation, Theme);
                        string InfoPath = Path.Combine(ThemePath, SMR.SucroseInfo);

                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                        {
                            SSTHI Info = SSTHI.ReadJson(InfoPath);

                            IEnumerable<string> SearchText = Info.Title.Split(' ')
                                       .Concat(Info.Description.Split(' '))
                                       .Concat(Info.Tags ?? Array.Empty<string>());

                            Searches.Add(Theme, string.Join(" ", SearchText.Select(Word => Word.ToLowerInvariant()).Distinct()));

                            if (SSDMM.LibrarySortMode == SSDESMT.Name)
                            {
                                SortThemes.Add(Theme, Info.Title);
                            }
                            else if (SSDMM.LibrarySortMode == SSDESMT.Creation)
                            {
                                SortThemes.Add(Theme, Directory.GetCreationTime(Path.Combine(SMMM.LibraryLocation, Theme)));
                            }
                            else if (SSDMM.LibrarySortMode == SSDESMT.Modification)
                            {
                                SortThemes.Add(Theme, File.GetLastWriteTime(InfoPath));
                            }
                        }
                    }

                    if (SortThemes != null && SortThemes.Any())
                    {
                        if (SSDMM.LibrarySortKind == SSDESKT.Ascending)
                        {
                            SortThemes = SortThemes.OrderBy(Theme => Theme.Value).ToDictionary(Theme => Theme.Key, Theme => Theme.Value);
                        }
                        else
                        {
                            SortThemes = SortThemes.OrderByDescending(Theme => Theme.Value).ToDictionary(Theme => Theme.Key, Theme => Theme.Value);
                        }

                        Themes.Clear();
                        Themes.AddRange(SortThemes.Select(Theme => Theme.Key));
                    }
                }
            }
            else
            {
                Themes.Clear();

                Directory.CreateDirectory(SMMM.LibraryLocation);
            }

            SMMI.ThemesSettingManager.SetSetting(SMC.Themes, Themes);
        }

        private async Task Start(bool Progress = false)
        {
            if (Themes != null && Themes.Any())
            {
                FullLibraryPage = new(Searches, Themes);

                FrameLibrary.Content = FullLibraryPage;
            }
            else
            {
                EmptyLibraryPage = new();

                FrameLibrary.Content = EmptyLibraryPage;
            }

            if (!Progress)
            {
                await Task.Delay(500);

                FrameLibrary.Visibility = Visibility.Visible;
                ProgressLibrary.Visibility = Visibility.Collapsed;
            }
        }

        private void GridLibrary_DragOver(object sender, DragEventArgs e)
        {
            if (!e.Data.GetDataPresent(DataFormats.FileDrop) || e.AllowedEffects.HasFlag(DragDropEffects.Copy) == false)
            {
                e.Effects = DragDropEffects.None;
            }
            else
            {
                e.Effects = DragDropEffects.Copy;
                DropRectangle.Visibility = Visibility.Visible;
            }
        }

        private void GridLibrary_DragLeave(object sender, DragEventArgs e)
        {
            DropRectangle.Visibility = Visibility.Collapsed;
        }

        private async void GridLibrary_Drop(object sender, DragEventArgs e)
        {
            DropRectangle.Visibility = Visibility.Collapsed;

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                bool State = false;
                List<SSDECT> Types = new();
                List<string> Messages = new();

                string[] Files = (string[])e.Data.GetData(DataFormats.FileDrop);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        SSDECT Result = SSZHA.Check(Record);

                        if (Result == SSDECT.Pass)
                        {
                            if (!Directory.Exists(SMMM.LibraryLocation))
                            {
                                Directory.CreateDirectory(SMMM.LibraryLocation);
                            }

                            string Name;

                            do
                            {
                                Name = SHG.GenerateString(SMMM.Chars, 25, SMR.Randomise);
                            } while (Directory.Exists(Path.Combine(SMMM.LibraryLocation, Name)));

                            Result = await Task.Run(() => SSZEZ.Extract(Record, Path.Combine(SMMM.LibraryLocation, Name)));

                            if (Result == SSDECT.Pass)
                            {
                                State = true;
                                Messages.Add(string.Format(SRER.GetValue("Portal", "LibraryPage", "Import", "Successful"), Path.GetFileNameWithoutExtension(Record)));
                            }
                            else
                            {
                                Messages.Add(string.Format(SRER.GetValue("Portal", "LibraryPage", "Import", "Unsuccessful"), Path.GetFileNameWithoutExtension(Record), Result));
                            }
                        }
                        else
                        {
                            if (!Types.Contains(Result))
                            {
                                Types.Add(Result);
                            }

                            Messages.Add(string.Format(SRER.GetValue("Portal", "LibraryPage", "Import", "Unsuccessful"), Path.GetFileNameWithoutExtension(Record), Result));
                        }
                    }
                }

                if (Messages.Any() || Types.Any())
                {
                    SPVCTI ThemeImport = new()
                    {
                        Types = Types,
                        Messages = Messages
                    };

                    await ThemeImport.ShowAsync();

                    ThemeImport.Dispose();
                }

                if (State)
                {
                    Dispose();

                    InitializeThemes();

                    await Start(true);
                }
            }
        }

        private async void GridLibrary_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }

        private async void SearchService_SearchTextChanged(object sender, EventArgs e)
        {
            if (string.IsNullOrEmpty(SPMI.SearchService.SearchText) || !string.IsNullOrWhiteSpace(SPMI.SearchService.SearchText))
            {
                Dispose();

                await Start(true);
            }
        }

        private async void LibraryService_CreatedWallpaper(object sender, EventArgs e)
        {
            Dispose();

            InitializeThemes();

            await Start(true);
        }

        public void Dispose()
        {
            FullLibraryPage?.Dispose();
            EmptyLibraryPage?.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}