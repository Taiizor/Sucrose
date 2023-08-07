using Sucrose.Portal.Views.Controls;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Media;
using Wpf.Ui.Common;

namespace Sucrose.Portal.Views.Pages.Setting
{
    /// <summary>
    /// GeneralSettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class GeneralSettingPage : Page
    {
        public GeneralSettingPage()
        {
            InitializeComponent();
        }

        private async Task Start()
        {
            //bir ayarlar dizisi oluştur ve ona uygun usercontrolü (controls) FrameSetting'e Children
            //olarak ekle

            TextBlock Tb1 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Foreground = Brushes.White,
                Text = "Görünüş & Davranış"
            };
            TextBlock Tb2 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Foreground = Brushes.White,
                Text = "Uygulama"
            };
            TextBlock Tb3 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Foreground = Brushes.White,
                Text = "Sistem"
            };

            ExpanderCard CustomExpander1 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            CustomExpander1.Title.Text = "Başlangıçta Çalıştır";
            CustomExpander1.LeftIcon.Symbol = SymbolRegular.Play20;
            CustomExpander1.Description.Text = "Duvar kağıdını oynatabilmek için Sucrose arka planda çalışmalı.";

            Wpf.Ui.Controls.ToggleSwitch TS1 = new() { Content = "Açık", IsChecked = true };

            CustomExpander1.HeaderFrame = TS1;

            ExpanderCard CustomExpander2 = new()
            {
                Margin = new Thickness(0, -30, 0, 0),
                Expandable = false
            };

            CustomExpander2.Title.Text = "Bildirim Alanı Simgesi";
            CustomExpander2.LeftIcon.Symbol = SymbolRegular.TrayItemAdd20;
            CustomExpander2.Description.Text = "Sistem tepsisinde ikon görünürlüğü, Sucrose ikon gizli bir şekilde çalışmaya devam edecek";

            ComboBox CB1 = new() { Width = 200 };

            CB1.Items.Add("Normal");
            CB1.Items.Add("Görünmez");

            CB1.SelectedIndex = 0;

            CustomExpander2.HeaderFrame = CB1;

            ExpanderCard CustomExpander3 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            CustomExpander3.Title.Text = "Ses Düzeyi";
            CustomExpander3.LeftIcon.Symbol = SymbolRegular.Speaker220;
            CustomExpander3.Description.Text = "Tüm duvar kağıtları için ses seviyesi";

            Slider Slider1 = new()
            {
                TickPlacement = TickPlacement.Both,
                IsSnapToTickEnabled = false,
                TickFrequency = 2,
                Width = 200,
                Value = 100
            };

            CustomExpander3.HeaderFrame = Slider1;

            CheckBox CB2 = new() { Content = "Sesi yalnızca masaüstü odaklandığında oynat", IsChecked = true };

            CustomExpander3.FooterCard = CB2;

            ExpanderCard CustomExpander4 = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true,
                IsExpand = true
            };

            CustomExpander4.LeftIcon.Width = 0;
            CustomExpander4.LeftIcon.Height = 0;
            CustomExpander4.Title.Text = "Video Oynatıcı";
            CustomExpander4.Description.Text = "Wıdeo duvar kağıdı oynatıcısını seçin";

            StackPanel SP1 = new()
            {
                Orientation = Orientation.Horizontal,
                HorizontalAlignment = HorizontalAlignment.Stretch,
                VerticalAlignment = VerticalAlignment.Stretch
            };

            TextBlock TB1 = new() { Text = "Testing" };

            SP1.Children.Add(TB1);

            CustomExpander4.FooterCard = SP1;

            FrameSetting.Children.Add(Tb1);

            FrameSetting.Children.Add(CustomExpander1);
            FrameSetting.Children.Add(CustomExpander2);

            FrameSetting.Children.Add(Tb2);

            FrameSetting.Children.Add(CustomExpander3);
            FrameSetting.Children.Add(CustomExpander4);

            FrameSetting.Children.Add(Tb3);


            ExpanderCard CustomExpander10 = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            CustomExpander10.LeftIcon.Symbol = SymbolRegular.Color20;
            CustomExpander10.LeftIcon.Filled = false;

            Slider Slider2 = new()
            {
                TickPlacement = TickPlacement.Both,
                IsSnapToTickEnabled = true,
                TickFrequency = 20,
                Width = 200,
                Value = 50
            };

            Slider2.ValueChanged += (s, e) => CustomExpander10.Expandable = !CustomExpander10.Expandable;

            CustomExpander10.HeaderFrame = Slider2;

            ComboBox CB10 = new();

            CB10.Items.Add("Test1");
            CB10.Items.Add("Test2");
            CB10.Items.Add("Test3");
            CB10.Items.Add("Test4");
            CB10.Items.Add("Test5");
            CB10.Items.Add("Test6");

            CB10.SelectedIndex = 0;

            CustomExpander10.FooterCard = CB10;

            FrameSetting.Children.Add(CustomExpander10);




            foreach (string List in new List<string>())
            {
                //

                await Task.Delay(25);
            }

            await Task.Delay(500);

            FrameSetting.Visibility = Visibility.Visible;
            ProgressSetting.Visibility = Visibility.Collapsed;
        }

        private async void GridSetting_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }
    }
}
