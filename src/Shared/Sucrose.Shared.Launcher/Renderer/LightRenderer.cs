﻿using System.Drawing.Drawing2D;
using SSLTLCT = Sucrose.Shared.Launcher.Table.LightColorTable;

namespace Sucrose.Shared.Launcher.Renderer
{
    public class LightRenderer : ToolStripProfessionalRenderer
    {
        public LightRenderer() : base(new SSLTLCT())
        {
        }

        protected override void OnRenderArrow(ToolStripArrowRenderEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            Rectangle Rectangle = new(e.ArrowRectangle.Location, e.ArrowRectangle.Size);

            Rectangle.Inflate(-2, -6);

            e.Graphics.DrawLines
            (
                Pens.White, new Point[]
                {
                    new(Rectangle.Left, Rectangle.Top),
                    new(Rectangle.Right, Rectangle.Top + (Rectangle.Height /2)),
                    new(Rectangle.Left, Rectangle.Top + Rectangle.Height)
                }
            );
        }

        protected override void OnRenderItemCheck(ToolStripItemImageRenderEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            Rectangle Rectangle = new(e.ImageRectangle.Location, e.ImageRectangle.Size);

            Rectangle.Inflate(-4, -6);

            e.Graphics.DrawLines
            (
                Pens.White, new Point[]
                {
                    new(Rectangle.Left, Rectangle.Bottom - (Rectangle.Height /2)),
                    new(Rectangle.Left + (Rectangle.Width /3),  Rectangle.Bottom),
                    new(Rectangle.Right, Rectangle.Top)
                }
            );
        }

        protected override void OnRenderItemText(ToolStripItemTextRenderEventArgs e)
        {
            e.Item.ForeColor = Color.Black;
            base.OnRenderItemText(e);
        }

        protected override void OnRenderMenuItemBackground(ToolStripItemRenderEventArgs e)
        {
            if (!e.Item.Selected)
            {
                base.OnRenderMenuItemBackground(e);
            }
            else
            {
                SolidBrush FillColor = new(Color.FromArgb(175, 30, 144, 255));
                Pen BorderColor = new(Color.FromArgb(30, 144, 255));

                Rectangle Rectangle = new(Point.Empty, e.Item.Size);

                e.Graphics.FillRectangle(FillColor, Rectangle);
                e.Graphics.DrawRectangle(BorderColor, 1, 0, Rectangle.Width - 2, Rectangle.Height - 1);

                FillColor.Dispose();
                BorderColor.Dispose();
            }
        }
    }
}