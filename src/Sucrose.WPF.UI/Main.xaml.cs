using System.Windows;
using System.Windows.Controls;

namespace Sucrose.WPF.UI
{
    /// <summary>
    /// Main.xaml etkileşim mantığı
    /// </summary>
    public partial class Main : Window
    {
        private readonly string Uri1 = "https://www.vegalya.com/3/0wj1biqk.f41/fluid.html";
        private readonly string Uri2 = "https://www.vegalya.com/3/iqdvd4pt.jyo/triangle.html";
        private readonly string Uri3 = "https://www.vegalya.com/3/lgz4xpht.sjn/index.html";
        private readonly string Uri4 = "https://www.vegalya.com/3/mrn3a0m5.0lk/index.html";
        private readonly string Uri5 = "https://www.vegalya.com/3/rtnm43pj.wyl/index.html";
        private readonly string Uri6 = "https://www.vegalya.com/3/xn0quq52.bq2/index.html";
        private readonly string Uri7 = "https://www.vegalya.com/3/wxqfmbno.3vk/index.html";
        private readonly string Uri8 = "https://www.vegalya.com/3/ptpcxvcd.hlz/index.html";
        private readonly string Uri9 = "https://www.vegalya.com/3/nps35xrp.5eh/jellyfish.html";

        public Main()
        {
            InitializeComponent();
        }

        private void Button_Click(object sender, RoutedEventArgs e)
        {
            switch ((sender as Button).Content)
            {
                case "Design 1":
                    //Engine = new Back(Uri1, true);
                    break;
                case "Design 2":
                    //Engine = new Back(Uri2, true);
                    break;
                case "Design 3":
                    //Engine = new Back(Uri3, false);
                    break;
                case "Design 4":
                    //Engine = new Back(Uri4, false);
                    break;
                case "Design 5":
                    //Engine = new Back(Uri5, false);
                    break;
                case "Design 6":
                    //Engine = new Back(Uri6, false);
                    break;
                case "Design 7":
                    //Engine = new Back(Uri7, false);
                    break;
                case "Design 8":
                    //Engine = new Back(Uri8, false);
                    break;
                case "Design 9":
                    //Engine = new Back(Uri9, false);
                    break;
                default:
                    break;
            }
        }
    }
}