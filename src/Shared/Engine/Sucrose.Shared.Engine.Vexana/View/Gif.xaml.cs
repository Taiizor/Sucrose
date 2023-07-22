using System.Windows;
using System.Windows.Media;
using SESEH = Sucrose.Engine.Shared.Event.Handler;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSEVEG = Sucrose.Shared.Engine.Vexana.Event.Gif;
using SSEVHG = Sucrose.Shared.Engine.Vexana.Helper.Gif;
using SSEVHP = Sucrose.Shared.Engine.Vexana.Helper.Parse;
using SSEVMI = Sucrose.Shared.Engine.Vexana.Manage.Internal;

namespace Sucrose.Shared.Engine.Vexana.View
{
    /// <summary>
    /// Interaction logic for Gif.xaml
    /// </summary>
    public sealed partial class Gif : Window
    {
        public Gif(string Gif)
        {
            InitializeComponent();

            ContentRendered += (s, e) => SESEH.ContentRendered(this);

            Content = SSEVMI.ImageEngine;

            SSEVMI.ImageResult = SSEVHP.Gif(SESHS.GetSource(Gif).ToString());

            SESMI.GeneralTimer.Tick += new EventHandler(GeneralTimer_Tick);
            SESMI.GeneralTimer.Interval = new TimeSpan(0, 0, 1);
            SESMI.GeneralTimer.Start();

            Closing += (s, e) => SSEVMI.ImageTimer.Stop();
            Loaded += (s, e) => SESEH.WindowLoaded(this);

            SSEVMI.ImageTimer.Tick += new EventHandler(SSEVEG.ImageTimer_Tick);
            SSEVMI.ImageTimer.Start();
        }

        private void GeneralTimer_Tick(object sender, EventArgs e)
        {
            SSEVHG.SetLoop(SESHD.GetLoop());

            SSEVMI.ImageEngine.Stretch = (Stretch)SESHD.GetStretch();
        }
    }
}