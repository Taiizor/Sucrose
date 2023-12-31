﻿using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHC = Sucrose.Shared.Space.Helper.Clean;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeShare.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeShare : ContentDialog, IDisposable
    {
        private readonly SPEIL Loader = new();
        internal string Theme = string.Empty;
        internal SSTHI Info = new();

        public ThemeShare() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private async void Export_Click(object sender, RoutedEventArgs e)
        {
            Export.IsEnabled = false;

            await Task.Run(() =>
            {
                SaveFileDialog SaveDialog = new()
                {
                    FileName = SSSHC.FileName(Info.Title),

                    Filter = SRER.GetValue("Portal", "ThemeShare", "SaveDialogFilter"),
                    FilterIndex = 1,

                    Title = SRER.GetValue("Portal", "ThemeShare", "SaveDialogTitle"),

                    InitialDirectory = SMR.DesktopPath
                };

                if (SaveDialog.ShowDialog() == true)
                {
                    string Destination = SaveDialog.FileName;

                    SSZEZ.Compress(Theme, Destination);
                }
            });

            Export.IsEnabled = true;
        }

        private void Publish_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Publish}{SMR.ValueSeparator}{SMR.StoreWebsite}");
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                ThemeThumbnail.Source = Loader.LoadOptimal(ImagePath);
            }

            ThemeTitle.Text = Info.Title;
            ThemeDescription.Text = Info.Description;
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}