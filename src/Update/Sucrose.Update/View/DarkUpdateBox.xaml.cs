﻿using System.Media;
using System.Windows;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWHWI = Skylark.Wing.Helper.WindowInterop;

namespace Sucrose.Update.View
{
    /// <summary>
    /// Interaction logic for DarkUpdateBox.xaml
    /// </summary>
    public partial class DarkUpdateBox : Window
    {
        private static string Path = string.Empty;

        public DarkUpdateBox(string BundlePath)
        {
            InitializeComponent();

            Path = BundlePath;

            SystemSounds.Asterisk.Play();

            SourceInitialized += DarkUpdateBox_SourceInitialized;
        }

        private async void InstallButton_Click(object sender, RoutedEventArgs e)
        {
            Install_Button.IsEnabled = false;

            await Task.Run(() => SSSHP.Run(Path));

            await Task.Delay(TimeSpan.FromSeconds(10));

            Install_Button.IsEnabled = true;
        }

        private void DarkUpdateBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }
    }
}