using System.IO;
using System.Windows;
using SHG = Skylark.Helper.Generator;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSIC = Sucrose.Shared.Store.Interface.Category;
using SSSIR = Sucrose.Shared.Store.Interface.Root;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// StorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class StorePage : IDisposable
    {
        private static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        private static string LibraryLocation => SMMI.EngineSettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public StorePage()
        {
            InitializeComponent();
        }

        private async Task Start()
        {
            string StoreFile = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SMR.StoreFile);

            if (SSSHD.Store(StoreFile, Agent, Key))
            {
                //MessageBox.Show(SSSHS.Json(StoreFile));

                SSSIR Root = SSSHS.DeserializeRoot(StoreFile);

                foreach (KeyValuePair<string, SSSIC> Category in Root.Categories)
                {
                    //MessageBox.Show("Category: " + Category.Key);

                    foreach (KeyValuePair<string, SSSIW> Wallpaper in Category.Value.Wallpapers)
                    {
                        //MessageBox.Show("Wallpaper: " + Wallpaper.Key);

                        //MessageBox.Show("Source: " + Wallpaper.Value.Source);
                        //MessageBox.Show("Cover: " + Wallpaper.Value.Cover);
                        //MessageBox.Show("Live: " + Wallpaper.Value.Live);

                        string Keys = SHG.GenerateString(Chars, 25, SMR.Randomise);
                        bool Result = SSSHD.Theme(Path.Combine(Wallpaper.Value.Source, Wallpaper.Key), Path.Combine(LibraryLocation, Keys), Agent, Keys, Key).Result;
                    }
                }
            }
        }

        private async void Page_Loaded(object sender, RoutedEventArgs e)
        {
            if (!Directory.Exists(LibraryLocation) || !Directory.GetFiles(LibraryLocation, "*", SearchOption.AllDirectories).Any())
            {
                await Task.Run(Start);

                State.Text = "Temalar İndirildi.";
            }
            else
            {
                State.Text = "Temalar İndirilmiş.";
            }

            Ring.Progress = 100;
            Ring.IsIndeterminate = false;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}