from flask import render_template, flash, redirect, url_for, request, abort, Blueprint

main = Blueprint("main", __name__)

@main.route("/home")
@main.route("/")
def home():
    return "Hi!"